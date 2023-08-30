use crate::script::config::{transform, KeyOrButton, Method, ScriptConfig};
use rdev::{listen, simulate, Event, EventType, ListenError};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::sleep;

pub mod config;

/// 脚本列表
pub struct ScriptList(pub Vec<Script>);

impl ScriptList {
    /// 加载脚本配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let data = fs::read_to_string(path)?;
        let data: ScriptConfig = toml::from_str(&data)?;
        let list: Vec<Script> = data
            .scripts
            .into_iter()
            .map(|item| Script {
                delay: item.delay.unwrap_or(data.delay),
                trigger: item.trigger.into_iter().map(|m| (m, false)).collect(),
                repeat: item.repeat,
                task: None,
                methods: Arc::new(transform(item.methods, data.scaling)),
            })
            .collect();

        Ok(Self(list))
    }

    /// 监听脚本的触发
    pub fn listening(mut self) -> Result<(), ListenError> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

        tokio::spawn(async move {
            let triggers: HashSet<KeyOrButton> = self
                .0
                .iter()
                .flat_map(|m| m.trigger.keys().cloned())
                .collect();

            while let Some(event) = rx.recv().await {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        let key = KeyOrButton::Key(key);
                        if triggers.contains(&key) {
                            for item in self.0.iter_mut() {
                                item.down(&key)
                            }
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let key = KeyOrButton::Key(key);
                        if triggers.contains(&key) {
                            for item in self.0.iter_mut() {
                                item.up(&key)
                            }
                        }
                    }
                    EventType::ButtonPress(button) => {
                        let button = KeyOrButton::Mouse(button);
                        if triggers.contains(&button) {
                            for item in self.0.iter_mut() {
                                item.down(&button)
                            }
                        }
                    }
                    EventType::ButtonRelease(button) => {
                        let button = KeyOrButton::Mouse(button);
                        if triggers.contains(&button) {
                            for item in self.0.iter_mut() {
                                item.up(&button)
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        listen(move |event| {
            let _ = tx.send(event);
        })
    }
}

#[derive(Debug)]
pub struct Script {
    pub delay: u64,
    pub repeat: usize,
    pub methods: Arc<Vec<Method>>,
    pub task: Option<JoinHandle<()>>,
    pub trigger: HashMap<KeyOrButton, bool>,
}

impl Script {
    pub fn run(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
            // 开关型不重启
            if self.repeat == 0 {
                return;
            }
        }

        let delay = self.delay;
        let repeat = self.repeat;
        let methods = self.methods.clone();
        let task = tokio::task::spawn(async move {
            if repeat == 0 {
                loop {
                    run_method(&methods, delay).await;
                }
            } else {
                for _ in 0..repeat {
                    run_method(&methods, delay).await;
                }
            }
        });

        self.task = Some(task)
    }

    pub fn down(&mut self, key: &KeyOrButton) {
        if let Some(k) = self.trigger.get_mut(key) {
            *k = true;

            if self.trigger.values().all(|&flag| flag) {
                self.run()
            }
        }
    }

    pub fn up(&mut self, key: &KeyOrButton) {
        if let Some(k) = self.trigger.get_mut(key) {
            *k = false;
        }
    }
}

/// 运行脚本方法
async fn run_method(methods: &Arc<Vec<Method>>, delay: u64) {
    for method in methods.iter() {
        match method {
            Method::Event(event_type) => {
                if let Err(err) = simulate(event_type) {
                    println!("事件 {event_type:?} 执行失败: {err}");
                }
                if let EventType::MouseMove { .. } = event_type {
                    sleep(Duration::from_micros(100)).await;
                } else {
                    sleep(Duration::from_millis(delay)).await;
                };
            }
            Method::Custom(c) => c.run().await,
        }
    }
}
