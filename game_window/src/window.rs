use crate::{
    renderer::calc_logical_and_physical_window_size,
    settings::Settings,
    types::{EventLoop, LogicalSize, PhysicalSize},
};
use common::consts::APP_NAME;
use gfx_hal::window::Extent2D;
use winit::window::{Window, WindowBuilder};

// So the settings can be serialized easily
pub struct GameWindow {
    window: Window,
    logical_size: LogicalSize,
    physical_size: PhysicalSize,
    maximized: bool,
    surface_extent: Extent2D,
}

impl GameWindow {
    pub fn new(settings: &Settings) -> (GameWindow, EventLoop) {
        let event_loop = EventLoop::new();

        let (logical_size, physical_size) =
            calc_logical_and_physical_window_size(&event_loop, settings);
        let maximized = settings.graphics().maximized();

        let window_builder = WindowBuilder::new()
            .with_title(APP_NAME)
            .with_inner_size(logical_size)
            .with_maximized(maximized);

        let window = window_builder
            .build(&event_loop)
            .expect("Could not create Window");

        let surface_extent = Extent2D {
            width: physical_size.width,
            height: physical_size.height,
        };

        let this = Self {
            window,
            logical_size,
            physical_size,
            maximized,
            surface_extent,
        };
        (this, event_loop)
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn logical_size(&self) -> &LogicalSize {
        &self.logical_size
    }

    pub fn physical_size(&self) -> &PhysicalSize {
        &self.physical_size
    }

    pub fn maximized(&self) -> bool {
        self.maximized
    }

    pub fn surface_extent(&mut self) -> &mut Extent2D {
        &mut self.surface_extent
    }
    pub fn update_surface_extent(&mut self, extent: Extent2D) {
        self.surface_extent = extent
    }
}
