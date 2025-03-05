use eframe::Error;
use med_arts::app::app::PharmacyApp;
mod app;

fn main() -> eframe::Result<(), Error> {
    //let options = eframe::NativeOptions::default();
    //
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(
            // Convert SVG to a specific size (32x32 is common)
            eframe::icon_data::from_png_bytes(include_bytes!("../pharmacy-payroll-icon.png"))
                .expect("Failed to load icon"),
        ),
        ..Default::default()
    };

    eframe::run_native(
        "Med Arts Payroll",
        options,
        Box::new(|_cc| Ok(Box::new(PharmacyApp::new()))),
    )
}
