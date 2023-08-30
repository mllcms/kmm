use crate::script::ScriptList;
use clap::Parser;
use rdev::{listen, Event, EventType, Key};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;

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
}

impl Args {
    pub async fn run(self) {
        match self {
            Args::Run(r) => {
                match ScriptList::load(r.config) {
                    Ok(scripts) => {
                        if let Err(err) = scripts.listening() {
                            println!("监听脚本触发失败: {err:?}");
                        }
                    }
                    Err(err) => println!("加载脚本配置失败: {err}"),
                }
                sleep(Duration::from_secs(30)).await;
            }

            Args::Event => {
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

            Args::Point => {
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
        }
    }
}

#[derive(Debug, Parser)]
pub struct Run {
    /// 配置文件所在路径
    config: PathBuf,
}
