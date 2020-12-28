use crate::{global_state::GlobalState, renderer, types::EventLoop};
use gfx_hal::window::Extent2D;
use renderer::{ResourceHolder, Resources};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

pub fn run(global_state: GlobalState, event_loop: EventLoop) {
    let mut should_configure_swapchain = true;
    let mut game_window = global_state.window;
    let mut resource_holder: ResourceHolder =
        ResourceHolder::new(&event_loop, &global_state.settings, game_window.window());

    let start_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(dims) => {
                    game_window.update_surface_extent(Extent2D {
                        width: dims.width,
                        height: dims.height,
                    });
                    should_configure_swapchain = true;
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    game_window.update_surface_extent(Extent2D {
                        width: new_inner_size.width,
                        height: new_inner_size.height,
                    });
                    should_configure_swapchain = true;
                }
                _ => (),
            },
            Event::MainEventsCleared => game_window.window().request_redraw(),
            Event::RedrawRequested(_) => {
                // Here's where we'll perform our rendering.

                let res: &mut Resources = &mut resource_holder.0;
                res.reset_fence_and_command_pool();
                if should_configure_swapchain {
                    res.reconfigure_swap(game_window.surface_extent());
                    should_configure_swapchain = false;
                }

                res.render(
                    &mut should_configure_swapchain,
                    game_window.surface_extent(),
                    start_time,
                );
            }
            _ => (),
        }
    });
}
