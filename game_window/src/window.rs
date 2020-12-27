use crate::{settings::Settings, types::EventLoop};
use common::consts::WINDOW_TITLE;
use winit::{
    dpi::LogicalSize,
    window::{Window, WindowBuilder},
};

// So the settings can be serialized easily
pub struct GameWindow {
    window: Window,
    size: [u16; 2],
    maximized: bool,
}

impl GameWindow {
    pub fn new(settings: &Settings) -> (GameWindow, EventLoop) {
        let event_loop = EventLoop::new();

        let size = settings.graphics().window_size();
        let maximized = settings.graphics().maximized();

        let window_builder = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(size[0], size[1]))
            .with_maximized(maximized);

        let window = window_builder
            .build(&event_loop)
            .expect("Could not create Window");

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
