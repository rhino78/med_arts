use eframe::Error;
use med_arts::app::app::PharmacyApp;
mod app;

fn main() -> eframe::Result<(), Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui app",
        options,
        Box::new(|_cc| Ok(Box::new(PharmacyApp::new()))),
    )
}
