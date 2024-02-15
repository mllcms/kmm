use std::{
    collections::{HashMap, HashSet},
    error::Error,
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
    pub blocks: HashMap<String, Vec<ScriptEvent>>,
    pub scripts: Vec<ScriptItem>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<(ScriptList, WindowList), Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        let mut config: Self = toml::from_str(&data)?;

        let hs: HashSet<&String> = config.scripts.iter().map(|m| &m.title).collect();

        if config.scripts.len() != hs.len() {
            return Err("title 不可重复".into());
        }

        let win = WindowList::init(config.point, config.font_size, config.font_color, config.border);

        let mut scripts = vec![];
        mem::swap(&mut config.scripts, &mut scripts);

        let list: Result<Vec<Script>, Box<dyn Error>> = scripts
            .into_iter()
            .map(|item| {
                Ok(Script {
                    title: Arc::new(item.title),
                    delay: item.delay.unwrap_or(config.delay),
                    trigger: item.trigger.into_iter().map(|m| (m, false)).collect(),
                    repeat: item.repeat,
                    task: None,
                    methods: Arc::new(config.transform(item.methods)?),
                    updater: win.updater.clone(),
                })
            })
            .collect();

        Ok((ScriptList(list?), win))
    }

    pub fn mouse_move(&self, x: f64, y: f64) -> EventType {
        let x = (x + self.offset.0) / self.scaling;
        let y = (y + self.offset.1) / self.scaling;
        EventType::MouseMove { x, y }
    }

    pub fn transform(&self, methods: Vec<ScriptEvent>) -> Result<Vec<Method>, Box<dyn Error>> {
        let mut res = Vec::new();
        for method in methods {
            match method {
                ScriptEvent::ClickDown(button) => res.push(Method::mouse_down(button)),
                ScriptEvent::ClickUp(button) => res.push(Method::mouse_up(button)),
                ScriptEvent::Click(button) => {
                    res.push(Method::mouse_down(button));
                    res.push(Method::mouse_up(button));
                }
                ScriptEvent::ClickOn(button, x, y) => {
                    res.push(Method::Event(self.mouse_move(x, y)));
                    res.push(Method::mouse_down(button));
                    res.push(Method::mouse_up(button));
                }
                ScriptEvent::ClickTo(button, x, y, x2, y2) => {
                    res.push(Method::Event(self.mouse_move(x, y)));
                    res.push(Method::mouse_down(button));
                    res.push(Method::Event(self.mouse_move(x2, y2)));
                    res.push(Method::mouse_up(button));
                }
                ScriptEvent::KeyDown(key) => res.push(Method::key_down(key)),
                ScriptEvent::KeyUp(key) => res.push(Method::key_up(key)),
                ScriptEvent::Key(key) => {
                    res.push(Method::key_down(key));
                    res.push(Method::key_up(key));
                }
                ScriptEvent::Keys(keys) => {
                    keys.iter().for_each(|key| res.push(Method::key_down(*key)));
                    keys.iter().for_each(|key| res.push(Method::key_up(*key)));
                }
                ScriptEvent::Scroll(delta_x, delta_y) => res.push(Method::Event(EventType::Wheel { delta_x, delta_y })),
                ScriptEvent::Move(x, y) => res.push(Method::Event(self.mouse_move(x, y))),
                ScriptEvent::Sleep(n) => res.push(Method::Custom(Custom::Sleep(n))),
                ScriptEvent::Exit => res.push(Method::Custom(Custom::Exit)),
                ScriptEvent::Block { sleep, repeat, block } => {
                    let block = match block {
                        Block::Name(name) => {
                            let block = self
                                .blocks
                                .get(&name)
                                .ok_or_else(|| format!("没有找到名为 {name:?} 的 block"))?
                                .to_owned();
                            if is_circular_reference(&block, &name) {
                                return Err(format!("不能引用自身同名的 {name:?} 的 block").into());
                            }
                            block
                        }
                        Block::Block(val) => val,
                    };
                    let block = self.transform(block)?;
                    for _ in 0..repeat {
                        res.extend(block.iter().cloned());
                        res.push(Method::Custom(Custom::Sleep(sleep)))
                    }
                    res.pop();
                }
            }
        }
        res.retain(|f| !matches!(f, Method::Custom(Custom::Sleep(0))));
        Ok(res)
    }
}

fn is_circular_reference(vec: &[ScriptEvent], name: &str) -> bool {
    vec.iter()
        .any(|a| matches!(a,ScriptEvent::Block { block: Block::Name(n), .. } if n == name))
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
#[serde(untagged)]
pub enum Block {
    Name(String),
    Block(Vec<ScriptEvent>),
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
    Block {
        repeat: usize,
        sleep: u64,
        block: Block,
    },
    Sleep(u64),
    Exit,
}

/// 自定义事件
#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Method {
    fn key_down(key: Key) -> Self {
        Self::Event(EventType::KeyPress(key))
    }
    fn key_up(key: Key) -> Self {
        Self::Event(EventType::KeyRelease(key))
    }
    fn mouse_down(button: Button) -> Self {
        Self::Event(EventType::ButtonPress(button))
    }
    fn mouse_up(button: Button) -> Self {
        Self::Event(EventType::ButtonRelease(button))
    }
}
