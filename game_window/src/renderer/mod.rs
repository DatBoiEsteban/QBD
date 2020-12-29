use crate::{
    settings::Settings,
    types::{EventLoop, LogicalSize, PhysicalSize},
};
use common::consts::APP_NAME;

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as back;
#[cfg(any(
    not(feature = "dx12"),
    not(feature = "gl"),
    not(feature = "metal"),
    not(feature = "vulkan")
))]
use gfx_backend_empty as back;
#[cfg(feature = "gl")]
use gfx_backend_gl as back;
#[cfg(feature = "metal")]
use gfx_backend_metal as back;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as back;

use gfx_hal::{
    adapter::Adapter,
    command::{ClearColor, ClearValue, CommandBuffer, CommandBufferFlags, Level, SubpassContents},
    device::Device,
    format::{ChannelType, Format},
    image::Layout,
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    prelude::{CommandQueue, QueueFamily},
    pso::ShaderStageFlags,
    queue::{QueueGroup, Submission},
    window::{Extent2D, PresentationSurface, Surface, SwapchainConfig},
    Instance,
};
use shaderc::ShaderKind;
use std::{mem::ManuallyDrop, time::Instant};
use winit::window::Window;

use self::push_constants::PushConstants;
mod push_constants;
pub struct Resources {
    pub instance: <back::Backend as gfx_hal::Backend>::Instance,
    pub adapter: Adapter<back::Backend>,
    pub surface: <back::Backend as gfx_hal::Backend>::Surface,
    pub device: <back::Backend as gfx_hal::Backend>::Device,
    pub render_passes: Vec<<back::Backend as gfx_hal::Backend>::RenderPass>,
    pub pipeline_layouts: Vec<<back::Backend as gfx_hal::Backend>::PipelineLayout>,
    pub pipelines: Vec<<back::Backend as gfx_hal::Backend>::GraphicsPipeline>,
    pub command_pool: <back::Backend as gfx_hal::Backend>::CommandPool,
    pub submission_complete_fence: <back::Backend as gfx_hal::Backend>::Fence,
    pub rendering_complete_semaphore: <back::Backend as gfx_hal::Backend>::Semaphore,
    pub surface_color_format: Format,
    pub command_buffer: <back::Backend as gfx_hal::Backend>::CommandBuffer,
    pub queue_group: QueueGroup<back::Backend>,
}

impl Resources {
    pub fn new(event_loop: &EventLoop, settings: &Settings, window: &Window) -> Resources {
        let (instance, adapter, surface) = generate_backend_instance(window);
        let (device, mut queue_group) = get_logical_device(&adapter, &surface);

        let (command_pool, mut command_buffer) = generate_command_buffer(&device, &queue_group);

        let surface_color_format = get_surface_color_format(&surface, &adapter);

        let render_pass = render_pass(surface_color_format, &device);

        let pipeline_layout = generate_pipeline(&device);

        let vertex_shader = include_str!("shaders/part-1.vert");
        let fragment_shader = include_str!("shaders/part-1.frag");

        let pipeline = unsafe {
            make_pipeline(
                &device,
                &render_pass,
                &pipeline_layout,
                vertex_shader,
                fragment_shader,
            )
        };

        let (submission_complete_fence, rendering_complete_semaphore) =
            generate_fence_and_semaphore(&device);
        Self {
            instance,
            adapter,
            surface,
            device,
            command_pool,
            render_passes: vec![render_pass],
            pipeline_layouts: vec![pipeline_layout],
            pipelines: vec![pipeline],
            submission_complete_fence,
            rendering_complete_semaphore,
            surface_color_format,
            command_buffer,
            queue_group,
        }
    }
    pub fn reset_fence_and_command_pool(&mut self) {
        let render_timeout_ns = 1_000_000_000;
        unsafe {
            self.device
                .wait_for_fence(&self.submission_complete_fence, render_timeout_ns)
                .expect("Out of memory or device lost");

            self.device
                .reset_fence(&self.submission_complete_fence)
                .expect("Out of memory");

            self.command_pool.reset(false);
        }
    }

    pub fn reconfigure_swap(&mut self, surface_extent: &mut Extent2D) {
        let caps = self.surface.capabilities(&self.adapter.physical_device);

        let mut swapchain_config =
            SwapchainConfig::from_caps(&caps, self.surface_color_format, *surface_extent);

        // This seems to fix some fullscreen slowdown on macOS.
        if caps.image_count.contains(&3) {
            swapchain_config.image_count = 3;
        }

        *surface_extent = swapchain_config.extent;

        unsafe {
            self.surface
                .configure_swapchain(&self.device, swapchain_config)
                .expect("Failed to configure swapchain");
        };
    }

    pub fn render(
        &mut self,
        should_configure_swapchain: &mut bool,
        surface_extent: &mut Extent2D,
        start_time: Instant,
    ) {
        let surface_image = unsafe {
            // We refuse to wait more than a second, to avoid hanging.
            let acquire_timeout_ns = 1_000_000_000;

            match self.surface.acquire_image(acquire_timeout_ns) {
                Ok((image, _)) => image,
                Err(_) => {
                    *should_configure_swapchain = true;
                    return;
                }
            }
        };

        let framebuffer = unsafe {
            use std::borrow::Borrow;

            use gfx_hal::image::Extent;

            self.device
                .create_framebuffer(
                    &self.render_passes[0],
                    vec![surface_image.borrow()],
                    Extent {
                        width: surface_extent.width,
                        height: surface_extent.height,
                        depth: 1,
                    },
                )
                .unwrap()
        };

        let viewport = {
            use gfx_hal::pso::{Rect, Viewport};

            Viewport {
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: surface_extent.width as i16,
                    h: surface_extent.height as i16,
                },
                depth: 0.0..1.0,
            }
        };
        unsafe {
            self.command_buffer
                .begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);

            self.command_buffer.set_viewports(0, &[viewport.clone()]);
            self.command_buffer.set_scissors(0, &[viewport.rect]);
            self.command_buffer.begin_render_pass(
                &self.render_passes[0],
                &framebuffer,
                viewport.rect,
                &[ClearValue {
                    color: ClearColor {
                        float32: [0.0, 0.0, 0.0, 1.0],
                    },
                }],
                SubpassContents::Inline,
            );
            self.command_buffer
                .bind_graphics_pipeline(&self.pipelines[0]);

            let anim = start_time.elapsed().as_secs_f32().sin() * 0.5 + 0.5;

            let small = [0.33, 0.33];

            let triangles = &[
                // Red triangle
                PushConstants {
                    color: [1.0, 0.0, 0.0, 1.0],
                    pos: [-0.5, -0.5],
                    scale: small,
                },
                // Green triangle
                PushConstants {
                    color: [0.0, 1.0, 0.0, 1.0],
                    pos: [0.0, -0.5],
                    scale: small,
                },
                // Blue triangle
                PushConstants {
                    color: [0.0, 0.0, 1.0, 1.0],
                    pos: [0.5, -0.5],
                    scale: small,
                },
                // Blue <-> cyan animated triangle
                PushConstants {
                    color: [0.0, anim, 1.0, 1.0],
                    pos: [-0.5, 0.5],
                    scale: small,
                },
                // Down <-> up animated triangle
                PushConstants {
                    color: [1.0, 1.0, 1.0, 1.0],
                    pos: [0.0, 0.5 - anim * 0.5],
                    scale: small,
                },
                // Small <-> big animated triangle
                PushConstants {
                    color: [1.0, 1.0, 1.0, 1.0],
                    pos: [0.5, 0.5],
                    scale: [0.33 + anim * 0.33, 0.33 + anim * 0.33],
                },
            ];
            for triangle in triangles {
                self.command_buffer.push_graphics_constants(
                    &self.pipeline_layouts[0],
                    ShaderStageFlags::VERTEX,
                    0,
                    push_constant_bytes(triangle),
                );

                self.command_buffer.draw(0..3, 0..1);
            }
            self.command_buffer.end_render_pass();
            self.command_buffer.finish();
        }

        unsafe {
            let submission = Submission {
                command_buffers: vec![&self.command_buffer],
                wait_semaphores: None,
                signal_semaphores: vec![&self.rendering_complete_semaphore],
            };

            self.queue_group.queues[0].submit(submission, Some(&self.submission_complete_fence));
            let result = self.queue_group.queues[0].present(
                &mut self.surface,
                surface_image,
                Some(&self.rendering_complete_semaphore),
            );

            *should_configure_swapchain |= result.is_err();

            self.device.destroy_framebuffer(framebuffer);
        }
    }
}

pub unsafe fn push_constant_bytes<T>(push_constants: &T) -> &[u32] {
    let size_in_bytes = std::mem::size_of::<T>();
    let size_in_u32s = size_in_bytes / std::mem::size_of::<u32>();
    let start_ptr = push_constants as *const T as *const u32;
    std::slice::from_raw_parts(start_ptr, size_in_u32s)
}

pub fn calc_logical_and_physical_window_size(
    event_loop: &EventLoop,
    settings: &Settings,
) -> (LogicalSize, PhysicalSize) {
    let dpi = event_loop.primary_monitor().unwrap().scale_factor();
    let logical: LogicalSize = settings.graphics().window_size().into();
    let physical: PhysicalSize = logical.to_physical(dpi);

    (logical, physical)
}

fn generate_backend_instance(
    window: &Window,
) -> (
    <back::Backend as gfx_hal::Backend>::Instance,
    Adapter<back::Backend>,
    <back::Backend as gfx_hal::Backend>::Surface,
) {
    let instance = back::Instance::create(APP_NAME, 1).expect("Backend not supported");

    let surface = unsafe {
        instance
            .create_surface(window)
            .expect("Failed to create surface for window")
    };

    let adapter = instance.enumerate_adapters().remove(0);

    (instance, adapter, surface)
}

fn get_logical_device(
    adapter: &Adapter<back::Backend>,
    surface: &<back::Backend as gfx_hal::Backend>::Surface,
) -> (
    <back::Backend as gfx_hal::Backend>::Device,
    QueueGroup<back::Backend>,
) {
    let queue_family = adapter
        .queue_families
        .iter()
        .find(|family| {
            surface.supports_queue_family(family) && family.queue_type().supports_graphics()
        })
        .expect("No compatible queue family found");

    let mut gpu = unsafe {
        use gfx_hal::adapter::PhysicalDevice;

        adapter
            .physical_device
            .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
            .expect("Failed to open device")
    };

    (gpu.device, gpu.queue_groups.pop().unwrap())
}

fn generate_command_buffer(
    device: &<back::Backend as gfx_hal::Backend>::Device,
    queue_group: &QueueGroup<back::Backend>,
) -> (
    <back::Backend as gfx_hal::Backend>::CommandPool,
    <back::Backend as gfx_hal::Backend>::CommandBuffer,
) {
    unsafe {
        let mut command_pool = device
            .create_command_pool(queue_group.family, CommandPoolCreateFlags::empty())
            .expect("Out of memory");

        let command_buffer = command_pool.allocate_one(Level::Primary);

        (command_pool, command_buffer)
    }
}

fn get_surface_color_format(
    surface: &<back::Backend as gfx_hal::Backend>::Surface,
    adapter: &Adapter<back::Backend>,
) -> Format {
    let supported_formats = surface
        .supported_formats(&adapter.physical_device)
        .unwrap_or_default();

    let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);

    supported_formats
        .into_iter()
        .find(|format| format.base_format().1 == ChannelType::Srgb)
        .unwrap_or(default_format)
}

fn render_pass(
    surface_color_format: Format,
    device: &<back::Backend as gfx_hal::Backend>::Device,
) -> <back::Backend as gfx_hal::Backend>::RenderPass {
    let color_attachment = Attachment {
        format: Some(surface_color_format),
        samples: 1,
        ops: AttachmentOps::new(AttachmentLoadOp::Clear, AttachmentStoreOp::Store),
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::Present,
    };

    let subpass = SubpassDesc {
        colors: &[(0, Layout::ColorAttachmentOptimal)],
        depth_stencil: None,
        inputs: &[],
        resolves: &[],
        preserves: &[],
    };

    unsafe {
        device
            .create_render_pass(&[color_attachment], &[subpass], &[])
            .expect("Out of memory")
    }
}

fn generate_pipeline(
    device: &<back::Backend as gfx_hal::Backend>::Device,
) -> <back::Backend as gfx_hal::Backend>::PipelineLayout {
    unsafe {
        let push_constant_bytes = std::mem::size_of::<push_constants::PushConstants>() as u32;
        device
            .create_pipeline_layout(&[], &[(ShaderStageFlags::VERTEX, 0..push_constant_bytes)])
            .expect("Out of memory")
    }
}

fn compile_shader(glsl: &str, shader_kind: ShaderKind) -> Vec<u32> {
    let mut compiler = shaderc::Compiler::new().unwrap();

    let compiled_shader = compiler
        .compile_into_spirv(glsl, shader_kind, "unnamed", "main", None)
        .expect("Failed to compile shader");

    compiled_shader.as_binary().to_vec()
}

/// # Safety
///
/// This shit se puede despichar, no se que hace but ok.
unsafe fn make_pipeline(
    device: &<back::Backend as gfx_hal::Backend>::Device,
    render_pass: &<back::Backend as gfx_hal::Backend>::RenderPass,
    pipeline_layout: &<back::Backend as gfx_hal::Backend>::PipelineLayout,
    vertex_shader: &str,
    fragment_shader: &str,
) -> <back::Backend as gfx_hal::Backend>::GraphicsPipeline {
    use gfx_hal::pass::Subpass;
    use gfx_hal::pso::{
        BlendState, ColorBlendDesc, ColorMask, EntryPoint, Face, GraphicsPipelineDesc,
        InputAssemblerDesc, Primitive, PrimitiveAssemblerDesc, Rasterizer, Specialization,
    };

    let vertex_shader_module = device
        .create_shader_module(&compile_shader(vertex_shader, ShaderKind::Vertex))
        .expect("Failed to create vertex shader module");

    let fragment_shader_module = device
        .create_shader_module(&compile_shader(fragment_shader, ShaderKind::Fragment))
        .expect("Failed to create fragment shader module");

    let (vs_entry, fs_entry) = (
        EntryPoint {
            entry: "main",
            module: &vertex_shader_module,
            specialization: Specialization::default(),
        },
        EntryPoint {
            entry: "main",
            module: &fragment_shader_module,
            specialization: Specialization::default(),
        },
    );
    let primitive_assembler = PrimitiveAssemblerDesc::Vertex {
        buffers: &[],
        attributes: &[],
        input_assembler: InputAssemblerDesc::new(Primitive::TriangleList),
        vertex: vs_entry,
        tessellation: None,
        geometry: None,
    };
    let mut pipeline_desc = GraphicsPipelineDesc::new(
        primitive_assembler,
        Rasterizer {
            cull_face: Face::BACK,
            ..Rasterizer::FILL
        },
        Some(fs_entry),
        pipeline_layout,
        Subpass {
            index: 0,
            main_pass: render_pass,
        },
    );

    pipeline_desc.blender.targets.push(ColorBlendDesc {
        mask: ColorMask::ALL,
        blend: Some(BlendState::ALPHA),
    });

    let pipeline = device
        .create_graphics_pipeline(&pipeline_desc, None)
        .expect("Failed to create graphics pipeline");

    device.destroy_shader_module(vertex_shader_module);
    device.destroy_shader_module(fragment_shader_module);

    pipeline
}

fn generate_fence_and_semaphore(
    device: &<back::Backend as gfx_hal::Backend>::Device,
) -> (
    <back::Backend as gfx_hal::Backend>::Fence,
    <back::Backend as gfx_hal::Backend>::Semaphore,
) {
    (
        device.create_fence(true).expect("Out of memory"),
        device.create_semaphore().expect("Out of memory"),
    )
}

pub struct ResourceHolder(pub ManuallyDrop<Resources>);
impl ResourceHolder {
    pub fn new(event_loop: &EventLoop, settings: &Settings, window: &Window) -> ResourceHolder {
        ResourceHolder(ManuallyDrop::new(Resources::new(
            event_loop, settings, window,
        )))
    }
}

impl Drop for ResourceHolder {
    fn drop(&mut self) {
        unsafe {
            let Resources {
                instance,
                mut surface,
                device,
                command_pool,
                render_passes,
                pipeline_layouts,
                pipelines,
                submission_complete_fence,
                rendering_complete_semaphore,
                adapter,
                surface_color_format,
                command_buffer,
                queue_group,
            } = ManuallyDrop::take(&mut self.0);

            device.destroy_semaphore(rendering_complete_semaphore);
            device.destroy_fence(submission_complete_fence);
            for pipeline in pipelines {
                device.destroy_graphics_pipeline(pipeline);
            }
            for pipeline_layout in pipeline_layouts {
                device.destroy_pipeline_layout(pipeline_layout);
            }
            for render_pass in render_passes {
                device.destroy_render_pass(render_pass);
            }
            device.destroy_command_pool(command_pool);
            surface.unconfigure_swapchain(&device);
            instance.destroy_surface(surface);
        }
    }
}
