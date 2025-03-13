use crate::app::app::PharmacyApp;
use eframe::egui;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub font_size: f32,
    pub button_size: [f32; 2],
    pub text_input_size: [f32; 2],
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            button_size: [200.0, 40.0],
            text_input_size: [200.0, 40.0],
        }
    }
}

impl UiSettings {
    pub fn load() -> Self {
        let config_dir = get_config_dir();
        let config_path = config_dir.join("settings.json");

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(json) => match serde_json::from_str(&json) {
                    Ok(settings) => return settings,
                    Err(e) => eprintln!("Error loading settings: {}", e),
                },
                Err(e) => eprintln!("Error loading settings: {}", e),
            }
        }

        let default_settings = UiSettings::default();

        if let Err(e) = default_settings.save() {
            eprintln!("Error saving settings: {}", e);
        }
        default_settings
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = get_config_dir();
        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("settings.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json)?;

        Ok(())
    }
}

fn get_config_dir() -> PathBuf {
    let mut config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.push("pharmacy_app");
    config_dir
}

pub fn render_settings(app: &mut PharmacyApp, ui: &mut egui::Ui) {
    ui.heading("Settings Panel");
    ui.label("Manage App settings here");
    ui.separator();

    let mut settings_modified = false;

    ui.label("Font Size");
    settings_modified |= ui
        .add(egui::Slider::new(&mut app.ui_settings.font_size, 12.0..=48.0).text("Font"))
        .changed();

    ui.separator();

    if ui.button("Save Settings").clicked() || settings_modified {
        if let Err(e) = app.ui_settings.save() {
            ui.colored_label(egui::Color32::RED, format!("Error saving settings: {}", e));
        } else {
            ui.colored_label(egui::Color32::GREEN, "Settings Saved Successfully");
        }
    }

    if ui.button("Reset to Defaults").clicked() {
        app.ui_settings = UiSettings::default();
        if let Err(e) = app.ui_settings.save() {
            ui.colored_label(egui::Color32::RED, format!("Error saving settings: {}", e));
        }
    }
}
