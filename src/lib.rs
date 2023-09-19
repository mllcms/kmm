use clap::Parser;
use rdev::{listen, Event, EventType, Key};
use std::ops::Sub;
use std::path::PathBuf;
use std::process::exit;
use std::time::{Duration, Instant};

use crate::script::config::{Config, ScriptEvent};

pub mod script;

/// 键鼠宏脚本(无反应或需 root 启动)
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub enum Args {
    /// 运行脚本
    Run(Run),
    /// 获取事件代码
    Event,
    /// 获取鼠标坐标 PS: Alt 输出当前坐标; Escape 清屏
    Point,
    /// 录制事件
    Record,
}

impl Args {
    pub async fn run(self) {
        match self {
            Args::Run(r) => r.run().await,
            Args::Event => event(),
            Args::Point => point(),
            Args::Record => record(),
        }
    }
}

#[derive(Debug, Parser)]
pub struct Run {
    /// 配置文件所在路径
    config: PathBuf,
}

impl Run {
    async fn run(self) {
        match Config::load(self.config) {
            Ok((script, window)) => {
                tokio::task::spawn_blocking(move || {
                    if let Err(err) = script.listening() {
                        println!("监听脚本触发失败: {err:?}");
                    }
                    std::thread::sleep(Duration::from_secs(30));
                });
                window.run().unwrap();
            }
            Err(err) => {
                println!("加载脚本配置失败: {err}");
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        };
    }
}

/// 获取事件代码
fn event() {
    fn callback(event: Event) {
        match event.event_type {
            EventType::KeyRelease(key) => {
                println!("🖮 -> {key:?}");
            }
            EventType::ButtonRelease(button) => {
                println!("🖰 -> {button:?}");
            }
            _ => {}
        }
    }
    let _ = listen(callback);
}

/// 获取坐标
fn point() {
    let mut point = (0.0, 0.0);
    let callback = move |event: Event| match event.event_type {
        EventType::MouseMove { x, y } => {
            point = (x, y);
        }
        EventType::KeyRelease(Key::AltGr) => {
            println!("{}, {}", point.0, point.1)
        }
        EventType::KeyRelease(Key::Escape) => {
            println!("\x1B[2J\x1B[1;1H");
        }
        _ => {}
    };
    let _ = listen(callback);
}

/// 录制事件
fn record() {
    let mut point = (0.0, 0.0);
    let mut prev = Instant::now();
    let mut res = vec![];
    let callback = move |event: Event| {
        if let EventType::MouseMove { x, y } = event.event_type {
            if x + y < 1_f64 {
                for item in res.iter() {
                    println!("{}", toml::to_string_pretty(&item).unwrap())
                }
                exit(0);
            }
            point = (x, y);
            return;
        }

        let curr = Instant::now();
        res.push(ScriptEvent::Sleep(curr.sub(prev).as_millis() as u64));
        prev = curr;

        match event.event_type {
            EventType::KeyPress(key) => res.push(ScriptEvent::KeyDown(key)),
            EventType::KeyRelease(key) => res.push(ScriptEvent::KeyUp(key)),
            EventType::ButtonPress(button) => {
                res.push(ScriptEvent::Move(point.0, point.1));
                res.push(ScriptEvent::ClickDown(button))
            }
            EventType::ButtonRelease(button) => {
                res.push(ScriptEvent::Move(point.0, point.1));
                res.push(ScriptEvent::ClickUp(button))
            }
            EventType::Wheel { delta_x, delta_y } => {
                res.push(ScriptEvent::Scroll(delta_x, delta_y))
            }
            _ => {}
        }
    };
    let _ = listen(callback);
}
