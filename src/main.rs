use env_logger::Env;

pub mod engine;
pub mod game;
pub mod render_state;
pub mod util;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .filter_module("forget_me_not", log::LevelFilter::Info)
        .init();

    pollster::block_on(game::run());
}
