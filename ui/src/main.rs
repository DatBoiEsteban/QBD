use ui::window::GameWindow;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
fn main() {
    let (window, event_loop) = GameWindow::new().expect("Failed to create Window");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.window.window().id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    })
}
