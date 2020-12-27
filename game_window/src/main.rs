use game_window::{global_state::GlobalState, run::run, settings::Settings, window::GameWindow};

fn main() {
    let settings = Settings::load();
    let (game_window, event_loop) = GameWindow::new(&settings);

    let game_state: GlobalState = GlobalState {
        settings,
        window: game_window,
    };

    run(game_state, event_loop);
}
