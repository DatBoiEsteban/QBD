use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use common::consts::WINDOW_TITLE;

// So the settings can be serialized easily
pub struct GameWindow {
    window: Window,
    size: [u16; 2],
    maximized: bool,
}

impl GameWindow {
    pub fn new() -> (GameWindow, EventLoop<()>) {
        let event_loop = EventLoop::new();

        let _size: [u16; 2] = [1280, 720]; // add as setting
        let _maximized = false; // add as setting

        let window_builder = WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(_size[0], _size[1]))
            .with_maximized(_maximized);

        let window = window_builder.build(&event_loop).unwrap();

        let this = Self {
            window: window,
            size: _size,
            maximized: _maximized,
        };
        (this, event_loop)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
