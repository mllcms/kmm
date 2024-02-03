use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use rdev::{listen, simulate, Event, EventType, ListenError};
use tokio::{
    sync::{mpsc, mpsc::UnboundedSender},
    task::JoinHandle,
    time::sleep,
};

use crate::script::config::{KeyOrButton, Method};

pub mod config;
pub mod window;

pub type Title = (Arc<String>, bool);

/// 脚本列表
pub struct ScriptList(pub Vec<Script>);

impl ScriptList {
    /// 监听脚本的触发
    pub fn listening(mut self) -> Result<(), ListenError> {
        let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

        tokio::spawn(async move {
            let triggers: HashSet<KeyOrButton> = self.0.iter().flat_map(|m| m.trigger.keys().cloned()).collect();

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
    pub title: Arc<String>,
    pub delay: u64,
    pub repeat: usize,
    pub methods: Arc<Vec<Method>>,
    pub task: Option<JoinHandle<()>>,
    pub trigger: HashMap<KeyOrButton, bool>,
    pub updater: UnboundedSender<Title>,
}

impl Script {
    pub fn run(&mut self) {
        let title = self.title.clone();
        let updater = self.updater.clone();

        if let Some(task) = self.task.take() {
            if self.repeat == 0 || !task.is_finished() {
                task.abort();
                return updater.send((title.clone(), false)).unwrap();
            }
        }

        let _ = updater.send((title.clone(), true));

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
                let _ = updater.send((title, false));
            }
        });

        self.task = Some(task);
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
