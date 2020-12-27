use crate::{global_state::GlobalState, types::EventLoop};
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
pub fn run(global_state: GlobalState, event_loop: EventLoop) {
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == global_state.window.window().id() => {
                *control_flow = ControlFlow::Exit
            }
            _ => (),
        }
    })
}
