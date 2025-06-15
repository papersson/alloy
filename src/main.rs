mod app;
mod core;
mod math;
mod renderer;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log!("Starting 3D Engine");
    App::run()
}
