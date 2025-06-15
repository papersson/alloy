mod app;
mod core;
mod input;
mod math;
mod renderer;
mod scene;
mod ui;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log!("Starting 3D Graphics Engine");
    App::run()
}
