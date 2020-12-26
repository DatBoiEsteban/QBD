use ui::window::GameWindow;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() {
    let (game_window, event_loop) = GameWindow::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == game_window.window().id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    })
}
