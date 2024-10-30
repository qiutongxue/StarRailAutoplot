use simple_logger::SimpleLogger;
use sr_plot_rs::plot::Plot;

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .with_module_level("xcap", log::LevelFilter::Off)
        .init()
        .unwrap();

    let image_files = vec![
        ("start.png", include_bytes!("../assets/start.png").to_vec()),
        (
            "start_ps5.png",
            include_bytes!("../assets/start_ps5.png").to_vec(),
        ),
        (
            "start_xbox.png",
            include_bytes!("../assets/start_xbox.png").to_vec(),
        ),
    ];

    let select_image = (
        "select.png",
        include_bytes!("../assets/select.png").to_vec(),
    );
    let plot = Plot::new("崩坏：星穹铁道".to_string(), select_image, image_files);
    plot.run();
}
