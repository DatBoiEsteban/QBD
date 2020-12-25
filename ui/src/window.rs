use glutin::{Api::OpenGl, ContextBuilder, ContextWrapper, GlRequest, PossiblyCurrent};
use old_school_gfx_glutin_ext::{ContextBuilderExt, WindowInitExt, WindowUpdateExt};
use serde::{Deserialize, Serialize};
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::ModifiersState,
    window::{Window, WindowBuilder},
};

pub type EventLoop = winit::event_loop::EventLoop<()>;
type ColorFormat = gfx::format::Srgba8;
type DepthFormat = gfx::format::DepthStencil;

pub struct GameWindow {
    pub window: ContextWrapper<PossiblyCurrent, Window>,
}
impl GameWindow {
    pub fn new() -> Result<(GameWindow, EventLoop), String> {
        let event_loop = EventLoop::new();
        let size: [u16; 2] = [1920, 1080];
        let win_builder = WindowBuilder::new()
            .with_title("QBD")
            .with_inner_size(LogicalSize::new(size[0] as f64, size[1] as f64))
            .with_maximized(true);

        let (window, device, factory, win_color_view, win_depth_view) = ContextBuilder::new()
            .with_gl(GlRequest::Specific(OpenGl, (3, 3)))
            .with_vsync(false)
            .with_gfx_color_depth::<ColorFormat, DepthFormat>()
            .build_windowed(win_builder, &event_loop)
            .map_err(|err| err.to_string())?
            .init_gfx::<ColorFormat, DepthFormat>();

        let mut this = Self { window: window };

        Ok((this, event_loop))
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum FullscreenMode {
    Exclusive,
    #[serde(other)]
    Borderless,
}

impl Default for FullscreenMode {
    fn default() -> Self {
        FullscreenMode::Borderless
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct FullScreenSettings {
    pub enabled: bool,
    pub mode: FullscreenMode,
    pub resolution: [u16; 2],
    pub bit_depth: Option<u16>,
    pub refresh_rate: Option<u16>,
}

impl Default for FullScreenSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: FullscreenMode::Borderless,
            resolution: [1920, 1080],
            bit_depth: None,
            refresh_rate: None,
        }
    }
}
