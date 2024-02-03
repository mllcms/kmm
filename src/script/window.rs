use std::{collections::HashMap, fmt::Write, sync::Arc};

use druid::{
    theme::TEXT_COLOR,
    widget::{CrossAxisAlignment, Flex, Label},
    *,
};
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::script::Title;

pub const MY_FONT: Key<FontDescriptor> = Key::new("my_font");

/// 显示运行中脚本的窗口
pub struct WindowList {
    pub app: AppLauncher<AppData>,
    pub app_data: AppData,
    pub updater: UnboundedSender<Title>,
}

impl WindowList {
    pub fn run(self) -> Result<(), PlatformError> {
        self.app.launch(self.app_data)
    }

    pub fn init(point: impl Into<Point>, font_size: f64, font_color: (u8, u8, u8), border: bool) -> Self {
        let (updater, mut rx) = mpsc::unbounded_channel::<Title>();
        let window = WindowDesc::new(ui_builder())
            .title("脚本列表")
            .set_position(point)
            .show_titlebar(border)
            .set_always_on_top(true)
            .transparent(true)
            .window_size_policy(WindowSizePolicy::Content);
        let app = AppLauncher::with_window(window).configure_env(move |env: &mut Env, _data: &AppData| {
            let new_font = FontDescriptor::new(FontFamily::SYSTEM_UI)
                .with_size(font_size)
                .with_weight(FontWeight::BLACK);
            env.set(MY_FONT, new_font);
            env.set(TEXT_COLOR, Color::rgb8(font_color.0, font_color.1, font_color.2));
        });

        let ext = app.get_external_handle();
        tokio::spawn(async move {
            while let Some((title, state)) = rx.recv().await {
                ext.add_idle_callback(move |data: &mut AppData| {
                    data.titles.insert(title, state);
                });
            }
        });

        Self { app, app_data: AppData::default(), updater }
    }
}

#[derive(Debug, Clone, Default, Data)]
pub struct AppData {
    #[data(eq)]
    pub titles: HashMap<Arc<String>, bool>,
}

fn ui_builder() -> impl Widget<AppData> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(
            Label::new(|data: &AppData, _: &_| {
                let mut s = String::new();
                for (title, state) in &data.titles {
                    if *state {
                        writeln!(&mut s, "{title}").unwrap();
                    }
                }
                s
            })
            .with_font(MY_FONT),
        )
        .background(Color::TRANSPARENT)
}
