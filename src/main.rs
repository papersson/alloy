mod app;
mod core;
mod math;
mod renderer;
mod scene;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log!("Starting 3D Engine");
    App::run()
}
