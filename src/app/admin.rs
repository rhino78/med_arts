use crate::app::app::PharmacyApp;

pub fn render_admin(app: &mut PharmacyApp, ui: &mut egui::Ui) {
    ui.heading("Admin Panel");
    ui.label("Manage administrative settings here.");
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Application Updates");
        if ui.button("Check for Updates").clicked() {
            app.check_for_update();
        }
    });

    app.render_update_status_detailed(ui);
}
