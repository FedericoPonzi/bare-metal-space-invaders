use env_logger::Env;
use log::info;
use space_invaders::{run_game, StdFrameBuffer};

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    run_game(StdFrameBuffer::new(), space_invaders::TimeManager::new());
    //while window.is_open() && !window.is_key_down(Key::Escape) {}
}
