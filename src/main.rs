mod app;
mod combat;
mod data;
mod ui;

use app::AoSApp;

fn main() {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1600.0, 1000.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AoS4 Combat Roller",
        options,
        Box::new(|cc| Ok(Box::new(AoSApp::new(cc)))),
    )
    .expect("Failed to start application");
}
