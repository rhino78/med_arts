use self::employee::Employee;
use crate::app::database;
use crate::app::employee::render_employees;
use crate::app::home::render_home;
use crate::app::settings::render_settings;
use crate::app::settings::UiSettings;
use crate::app::update::perform_update;
use chrono::NaiveDate;
use poll_promise::Promise;

pub use super::employee;
use crate::app::admin::render_admin;
use crate::app::payroll::render_payroll;
use crate::app::update::check_for_updates_blocking;
use chrono::Datelike;
use eframe::egui;
use rusqlite::Connection;

#[derive(PartialEq)]
pub enum ActivePanel {
    Home = 0,
    Admin = 1,
    Payroll = 2,
    Employees = 3,
    Settings = 4,
}

#[allow(dead_code)]
pub struct PharmacyApp {
    pub active_panel: ActivePanel,
    pub admin_text: String,
    pub employee_name: String,
    pub employee_position: String,
    pub search_name: String,
    pub search_result: Option<Employee>,
    pub search_status: String,
    pub conn: Connection,
    pub selected_employee: Option<Employee>,
    pub employees: Vec<Employee>,
    pub payroll_entries: [f32; 7],
    pub address: String,
    pub city: String,
    pub state: String,
    pub phone: String,
    pub filing_status: String,
    pub dependents: String,

    pub hours_worked: f32,
    pub withholding: f32,
    pub roth_ira: f32,
    pub social_security: f32,
    pub selected_friday: String,
    pub pay_rate: String,

    pub update_check:
        Option<Promise<Result<(Option<String>, String), Box<dyn std::error::Error + Send + Sync>>>>,
    pub update_available: Option<String>,
    pub release_notes: Option<String>,
    pub update_error: Option<String>,
    pub selected_employee_id: Option<i32>,
    pub selected_employee_index: usize,
    pub show_add_employee_popup: bool,
    pub gross: f32,
    pub net: f32,

    pub ui_settings: UiSettings,
}

impl Default for PharmacyApp {
    fn default() -> Self {
        Self::new()
    }
}

impl PharmacyApp {
    pub fn new() -> Self {
        let db_path = database::get_db_path();
        let conn = rusqlite::Connection::open(&db_path).unwrap_or_else(|e| {
            eprintln!("Database error: {}", e);
            std::process::exit(1);
        });

        database::initialize_tables(&conn).expect("Failed to initialize tables");

        let fridays = get_fridays_of_year();
        let selected_friday = fridays.first().cloned().unwrap_or_default();

        let ui_settings = UiSettings::load();

        let mut app = Self {
            active_panel: ActivePanel::Home,
            admin_text: String::new(),
            employee_name: String::new(),
            employee_position: String::new(),
            search_name: String::new(),
            search_result: None,
            search_status: String::new(),
            conn,
            employees: Vec::new(),
            selected_employee: None,
            payroll_entries: [0.0; 7],
            address: String::new(),
            city: String::new(),
            state: String::new(),
            phone: String::new(),
            filing_status: String::new(),
            dependents: String::new(),
            pay_rate: String::new(),

            hours_worked: 0.0,
            withholding: 0.0,
            roth_ira: 0.0,
            social_security: 0.0,
            selected_friday,
            net: 0.0,
            gross: 0.0,
            update_check: None,
            update_available: None,
            update_error: None,
            selected_employee_id: None,
            show_add_employee_popup: false,
            selected_employee_index: 0,
            release_notes: None,

            ui_settings,
        };

        app.employees = database::get_all_employees(&app.conn).expect("Failed to get employees");
        app
    }

    pub fn apply_text_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(self.ui_settings.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(self.ui_settings.font_size, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(self.ui_settings.font_size, egui::FontFamily::Proportional),
        );

        ctx.set_style(style);
    }

    pub fn check_for_update(&mut self) {
        self.update_check = Some(Promise::spawn_thread("update_check", || {
            check_for_updates_blocking()
        }));
    }

    pub fn render_update_status_detailed(&mut self, ui: &mut egui::Ui) {
        if let Some(promise) = &self.update_check {
            if let Some(result) = promise.ready() {
                match result {
                    Ok((Some(version), notes)) => {
                        self.update_available = Some(version.to_string());
                        self.release_notes = Some(notes.to_string());
                        self.update_error = None;
                    }
                    Ok((None, notes)) => {
                        self.update_error = None;
                        self.update_available = None;
                        self.release_notes = Some(notes.to_string());
                    }

                    Err(e) => {
                        self.update_error = Some(e.to_string());
                        self.update_available = None;
                        self.release_notes = None;
                    }
                }
            } else {
                ui.spinner();
                ui.label("Checking for Updates");
                return;
            }

            if let Some(error) = &self.update_error {
                ui.colored_label(egui::Color32::RED, format!("Update Error: {}", error));
            } else if let Some(version) = &self.update_available {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::LIGHT_GREEN,
                        format!("Update Available: {}", version),
                    );
                    if ui.button("Download").clicked() {
                        let _ = perform_update();
                    }
                });
            } else {
                ui.label("You are running the latest version");
            }

            if let Some(notes) = &self.release_notes {
                ui.separator();
                ui.label("Release notes");
                ui.label(notes);
            }
        }
    }

    pub fn render_update_status_brief(&mut self, ui: &mut egui::Ui) {
        if let Some(promise) = &self.update_check {
            if let Some(_result) = promise.ready() {
                if promise.ready().is_none() {
                    ui.spinner();
                    ui.label("Checking for Updates");
                    return;
                }
            }

            if let Some(_error) = &self.update_error {
                ui.colored_label(egui::Color32::RED, "Update Error");
            } else if let Some(version) = &self.update_available {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::LIGHT_GREEN,
                        format!("Update Available: {}", version),
                    );
                    if ui.button("Download").clicked() {
                        let _ = perform_update();
                    }
                });
            } else {
                ui.label("Latest version");
            }
        }
    }

    pub fn refresh_available_fridays(&mut self) {
        if self.selected_employee_index < self.employees.len() {
            let employee_id = self.employees[self.selected_employee_index].id;
            let available_fridays = get_available_fridays(&self.conn, employee_id);
            if !available_fridays.is_empty() {
                self.selected_friday = available_fridays[0].clone();
            }
        }
    }
}

pub fn get_available_fridays(conn: &Connection, employee_id: i32) -> Vec<String> {
    let all_fridays = get_fridays_of_year();
    let used_dates = match database::get_payroll_dates_for_employee(conn, employee_id) {
        Ok(dates) => dates,
        Err(_) => Vec::new(),
    };

    all_fridays
        .into_iter()
        .filter(|date| !used_dates.contains(date))
        .collect()
}

pub fn get_fridays_of_year() -> Vec<String> {
    let today = chrono::Local::now().date_naive();
    let year = today.year();
    let mut fridays = Vec::new();

    for month in 1..=12 {
        for day in 1..=31 {
            if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
                if date.weekday() == chrono::Weekday::Fri {
                    fridays.push(date.format("%Y-%m-%d").to_string());
                }
            }
        }
    }
    fridays
}

impl eframe::App for PharmacyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_text_style(ctx);
        // Create the top bar with buttons
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Admin").clicked() {
                    self.active_panel = ActivePanel::Admin;
                }
                if ui.button("Payroll").clicked() {
                    self.active_panel = ActivePanel::Payroll;
                }
                if ui.button("Employees").clicked() {
                    self.active_panel = ActivePanel::Employees;
                }
                if ui.button("Home").clicked() {
                    self.active_panel = ActivePanel::Home;
                }
                if ui.button("⚙ Settings").clicked() {
                    self.active_panel = ActivePanel::Settings;
                }
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_update_status_brief(ui);
            });
        });

        // Dynamically change the entire CentralPanel based on the selected button
        egui::CentralPanel::default().show(ctx, |ui| match self.active_panel {
            ActivePanel::Home => render_home(ui),
            ActivePanel::Admin => render_admin(self, ui),
            ActivePanel::Payroll => render_payroll(self, ui),
            ActivePanel::Employees => render_employees(self, ui),
            ActivePanel::Settings => render_settings(self, ui),
        });
    }
}
