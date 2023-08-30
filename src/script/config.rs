use rdev::{Button, EventType, Key};
use serde::{Deserialize, Serialize};
use std::process::exit;
use std::time::Duration;
use tokio::time::sleep;

/// 脚本配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptConfig {
    /// 延迟
    pub delay: u64,
    /// 缩放
    pub scaling: f64,
    pub scripts: Vec<ScriptItem>,
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
    /// 循环次数
    pub repeat: usize,

    /// 触发按键
    pub trigger: Vec<KeyOrButton>,

    /// 单独配置延迟
    pub delay: Option<u64>,

    /// 脚本方法
    pub methods: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "args")]
pub enum Event {
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

/// 事件转成脚本方法
pub fn transform(methods: Vec<Event>, scaling: f64) -> Vec<Method> {
    let mut res = Vec::new();
    for method in methods {
        match method {
            Event::ClickDown(button) => {
                res.push(Method::Event(EventType::ButtonPress(button)));
            }
            Event::ClickUp(button) => {
                res.push(Method::Event(EventType::ButtonRelease(button)));
            }
            Event::Click(button) => {
                res.push(Method::Event(EventType::ButtonPress(button)));
                res.push(Method::Event(EventType::ButtonRelease(button)));
            }
            Event::ClickOn(button, x, y) => {
                let (x, y) = (x / scaling, y / scaling);
                res.push(Method::Event(EventType::MouseMove { x, y }));
                res.push(Method::Event(EventType::ButtonPress(button)));
                res.push(Method::Event(EventType::ButtonRelease(button)));
            }
            Event::ClickTo(button, x, y, x2, y2) => {
                let (x, y, x2, y2) = (x / scaling, y / scaling, x2 / scaling, y2 / scaling);
                res.push(Method::Event(EventType::MouseMove { x, y }));
                res.push(Method::Event(EventType::ButtonPress(button)));
                res.push(Method::Event(EventType::MouseMove { x: x2, y: y2 }));
                res.push(Method::Event(EventType::ButtonRelease(button)));
            }
            Event::KeyUp(key) => {
                res.push(Method::Event(EventType::KeyRelease(key)));
            }
            Event::KeyDown(key) => {
                res.push(Method::Event(EventType::KeyPress(key)));
            }
            Event::Key(key) => {
                res.push(Method::Event(EventType::KeyPress(key)));
                res.push(Method::Event(EventType::KeyRelease(key)));
            }
            Event::Keys(keys) => {
                for key in &keys {
                    res.push(Method::Event(EventType::KeyPress(*key)));
                }
                for key in &keys {
                    res.push(Method::Event(EventType::KeyRelease(*key)));
                }
            }
            Event::Move(x, y) => {
                let (x, y) = (x / scaling, y / scaling);
                res.push(Method::Event(EventType::MouseMove { x, y }));
            }
            Event::Scroll(delta_x, delta_y) => {
                res.push(Method::Event(EventType::Wheel { delta_x, delta_y }));
            }
            Event::Sleep(n) => {
                res.push(Method::Custom(Custom::Sleep(n)));
            }
            Event::Exit => res.push(Method::Custom(Custom::Exit)),
        }
    }
    res
}
