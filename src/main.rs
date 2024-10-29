use std::env;

use simple_logger::SimpleLogger;
use sr_plot_rs::plot::Plot;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .with_module_level("xcap", log::LevelFilter::Off)
        .init()
        .unwrap();

    let image_files = ["start.png", "start_ps5.png", "start_xbox.png"];

    let plot = Plot::new(
        "崩坏：星穹铁道".to_string(),
        resource_path("select.png"),
        load_images(&image_files),
    );
    plot.run();
}

fn load_images(image_files: &[&str]) -> Vec<String> {
    image_files.iter().map(|path| resource_path(path)).collect()
}

fn resource_path(relative_path: &str) -> String {
    let path = env::current_dir()
        .unwrap()
        .join("assets")
        .join(relative_path);
    path.to_str().unwrap().to_string()
}
