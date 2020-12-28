#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PushConstants {
    pub color: [f32; 4],
    pub pos: [f32; 2],
    pub scale: [f32; 2],
}
