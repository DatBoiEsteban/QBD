use crate::settings::Settings;
use common::consts::WINDOW_TITLE;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

// So the settings can be serialized easily
pub struct GameWindow {
    window: Window,
    size: [u16; 2],
    maximized: bool,
}

impl GameWindow {
    pub fn new() -> (GameWindow, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let settings = Settings::load();

        let size = settings.graphics().window_size();
        let maximized = settings.graphics().maximized();

        let window_builder = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(size[0], size[1]))
            .with_maximized(maximized);

        let window = window_builder.build(&event_loop).unwrap();

        let this = Self {
            window,
            size,
            maximized,
        };
        (this, event_loop)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn size(&self) -> [u16; 2] {
        self.size
    }

    pub fn maximized(&self) -> bool {
        self.maximized
    }
}
