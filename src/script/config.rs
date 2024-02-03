use std::{
    collections::HashSet,
    error::Error,
    fmt::{Display, Formatter},
    fs, mem,
    path::Path,
    process::exit,
    sync::Arc,
    time::Duration,
};

use rdev::{Button, EventType, Key};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::script::{window::WindowList, Script, ScriptList};

/// 脚本配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 延迟
    pub delay: u64,
    /// 缩放
    pub scaling: f64,
    /// 偏移位置
    pub offset: (f64, f64),
    /// 窗口位置
    pub point: (f64, f64),
    /// 字体大小
    pub font_size: f64,
    /// 字体颜色
    pub font_color: (u8, u8, u8),
    /// 是否显示边框
    pub border: bool,
    pub scripts: Vec<ScriptItem>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<(ScriptList, WindowList), Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&data)?;

        let hs: HashSet<&String> = config.scripts.iter().map(|m| &m.title).collect();

        if config.scripts.len() != hs.len() {
            println!("title 不可重复");
            exit(0)
        }

        let win = WindowList::init(config.point, config.font_size, config.font_color, config.border);

        let mut scripts = vec![];
        mem::swap(&mut config.scripts, &mut scripts);

        let list: Vec<Script> = scripts
            .into_iter()
            .map(|item| Script {
                title: Arc::new(item.title),
                delay: item.delay.unwrap_or(config.delay),
                trigger: item.trigger.into_iter().map(|m| (m, false)).collect(),
                repeat: item.repeat,
                task: None,
                methods: Arc::new(config.transform(item.methods)),
                updater: win.updater.clone(),
            })
            .collect();

        Ok((ScriptList(list), win))
    }

    /// 修正坐标
    pub fn correct(&self, x: f64, y: f64) -> (f64, f64) {
        let x = (x + self.offset.0) / self.scaling;
        let y = (y + self.offset.1) / self.scaling;
        (x, y)
    }

    pub fn transform(&self, methods: Vec<ScriptEvent>) -> Vec<Method> {
        let mut res = Vec::new();
        for method in methods {
            match method {
                ScriptEvent::ClickDown(button) => {
                    res.push(Method::Event(EventType::ButtonPress(button)));
                }
                ScriptEvent::ClickUp(button) => {
                    res.push(Method::Event(EventType::ButtonRelease(button)));
                }
                ScriptEvent::Click(button) => {
                    res.push(Method::Event(EventType::ButtonPress(button)));
                    res.push(Method::Event(EventType::ButtonRelease(button)));
                }
                ScriptEvent::ClickOn(button, x, y) => {
                    let (x, y) = self.correct(x, y);
                    res.push(Method::Event(EventType::MouseMove { x, y }));
                    res.push(Method::Event(EventType::ButtonPress(button)));
                    res.push(Method::Event(EventType::ButtonRelease(button)));
                }
                ScriptEvent::ClickTo(button, x, y, x2, y2) => {
                    let (x, y) = self.correct(x, y);
                    let (x2, y2) = self.correct(x2, y2);
                    res.push(Method::Event(EventType::MouseMove { x, y }));
                    res.push(Method::Event(EventType::ButtonPress(button)));
                    res.push(Method::Event(EventType::MouseMove { x: x2, y: y2 }));
                    res.push(Method::Event(EventType::ButtonRelease(button)));
                }
                ScriptEvent::KeyUp(key) => {
                    res.push(Method::Event(EventType::KeyRelease(key)));
                }
                ScriptEvent::KeyDown(key) => {
                    res.push(Method::Event(EventType::KeyPress(key)));
                }
                ScriptEvent::Key(key) => {
                    res.push(Method::Event(EventType::KeyPress(key)));
                    res.push(Method::Event(EventType::KeyRelease(key)));
                }
                ScriptEvent::Keys(keys) => {
                    for key in &keys {
                        res.push(Method::Event(EventType::KeyPress(*key)));
                    }
                    for key in &keys {
                        res.push(Method::Event(EventType::KeyRelease(*key)));
                    }
                }
                ScriptEvent::Move(x, y) => {
                    let (x, y) = self.correct(x, y);
                    res.push(Method::Event(EventType::MouseMove { x, y }));
                }
                ScriptEvent::Scroll(delta_x, delta_y) => {
                    res.push(Method::Event(EventType::Wheel { delta_x, delta_y }));
                }
                ScriptEvent::Sleep(n) => {
                    res.push(Method::Custom(Custom::Sleep(n)));
                }
                ScriptEvent::Exit => res.push(Method::Custom(Custom::Exit)),
            }
        }
        res
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyOrButton {
    Key(Key),
    Mouse(Button),
}

/// 脚本每一项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptItem {
    /// 脚本标题
    pub title: String,

    /// 循环次数
    pub repeat: usize,

    /// 触发按键
    pub trigger: Vec<KeyOrButton>,

    /// 单独配置延迟
    pub delay: Option<u64>,

    /// 脚本方法
    pub methods: Vec<ScriptEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "args")]
pub enum ScriptEvent {
    /// 鼠标点击
    Click(Button),

    /// 鼠标松开
    ClickUp(Button),

    /// 鼠标按下
    ClickDown(Button),

    /// 点击指定位置
    ClickOn(Button, f64, f64),

    /// 拖拽到指定位置
    ClickTo(Button, f64, f64, f64, f64),

    /// 键盘松开
    KeyUp(Key),

    /// 键盘按下
    KeyDown(Key),

    /// 触发单按键
    Key(Key),

    /// 触发多个按键
    Keys(Vec<Key>),

    /// 移动鼠标到指定位置
    Move(f64, f64),

    /// 滚轮
    Scroll(i64, i64),

    /// 自定义事件
    Sleep(u64),
    Exit,
}

fn button_to_toml(button: &Button) -> String {
    match button {
        Button::Unknown(n) => format!("{{ Unknown = {n} }}"),
        _ => format!("{button:?}"),
    }
}

fn key_to_toml(key: &Key) -> String {
    match key {
        Key::Unknown(n) => format!("{{ Unknown = {n} }}"),
        _ => format!("{key:?}"),
    }
}

impl Display for ScriptEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ScriptEvent::Click(button) => button_to_toml(button),
            ScriptEvent::ClickUp(button) => button_to_toml(button),
            ScriptEvent::ClickDown(button) => button_to_toml(button),
            ScriptEvent::ClickOn(button, x, y) => {
                format!("{}, {}, {}", button_to_toml(button), x, y)
            }
            ScriptEvent::ClickTo(button, x, y, x2, y2) => {
                format!("{}, {x}, {y}, {x2}, {y2}", button_to_toml(button))
            }
            ScriptEvent::KeyUp(key) => key_to_toml(key),
            ScriptEvent::KeyDown(key) => key_to_toml(key),
            ScriptEvent::Key(key) => key_to_toml(key),
            ScriptEvent::Keys(keys) => format!("[{}]", keys.iter().map(key_to_toml).collect::<Vec<_>>().join(", ")),
            ScriptEvent::Move(x, y) => format!("[{x}, {y}]"),
            ScriptEvent::Scroll(x, y) => format!("[{x}, {y}]"),
            ScriptEvent::Sleep(n) => n.to_string(),
            _ => {
                let _ = write!(f, "{{ event = {:?} }}", self);
                return Ok(());
            }
        };
        write!(f, "{s}")
    }
}

/// 自定义事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "args")]
pub enum Custom {
    /// 睡眠 ms 毫秒
    Sleep(u64),

    /// 退出
    Exit,
}

impl Custom {
    pub async fn run(&self) {
        match self {
            Custom::Sleep(n) => sleep(Duration::from_millis(*n)).await,
            Custom::Exit => exit(0),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Method {
    /// 事件
    Event(EventType),
    /// 自定义
    Custom(Custom),
}
