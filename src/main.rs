mod app;
mod render_state;

fn main() {
    env_logger::init();
    pollster::block_on(app::run());
}
