use std::result;

use self::employee::Employee;
use crate::app::database;
use crate::app::stockservice;
use crate::app::ui::render_employees;
use chrono::NaiveDate;
use poll_promise::Promise;

pub use super::employee;
use crate::app::ui::render_payroll;
use crate::app::update::check_for_updates_blocking;
use crate::app::update::perform_update;
use chrono::Datelike;
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use plotters::prelude::*;
//use plotters_backend::DrawingBackend;
use rusqlite::params;
use rusqlite::Connection;

#[derive(PartialEq)]
pub enum ActivePanel {
    Home = 0,
    Admin = 1,
    Payroll = 2,
    Employees = 3,
}

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
    pub net: f32,
    pub gross: f32,
    pub pay_rate: String,

    update_check:
        Option<Promise<Result<Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>>>>,
    update_available: Option<String>,
    update_error: Option<String>,
    pub selected_employee_id: Option<i32>,
    //selected_employee: Option<Employee>,
    pub show_add_employee_popup: bool,
}

impl Default for PharmacyApp {
    fn default() -> Self {
        Self::new()
    }
}

impl PharmacyApp {
    pub fn get_all_employees(&self) -> result::Result<Vec<Employee>, rusqlite::Error> {
        database::get_all_employees(&self.conn)
    }

    pub fn calculate_gross(&self, hours_worked: f32, pay_rate: f32) -> f32 {
        hours_worked * pay_rate
    }

    pub fn calculate_withholding(&self, gross: f32) -> f32 {
        gross * 0.2
    }

    pub fn calculate_social_security(&self, gross: f32) -> f32 {
        gross * 0.075
    }

    pub fn calculate_net(&self, gross: f32, withholding: f32, social_security: f32) -> f32 {
        gross - withholding - social_security
    }

    pub fn new() -> Self {
        let db_path = database::get_db_path();
        let conn = rusqlite::Connection::open(&db_path).unwrap_or_else(|e| {
            eprintln!("Database error: {}", e);
            std::process::exit(1);
        });

        database::initialize_tables(&conn).expect("Failed to initialize tables");

        let fridays = get_fridays_of_year();
        let selected_friday = fridays.first().cloned().unwrap_or_default();

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
        };

        app.employees = database::get_all_employees(&app.conn).expect("Failed to get employees");
        app
    }

    fn check_for_update(&mut self) {
        self.update_check = Some(Promise::spawn_thread("update_check", || {
            check_for_updates_blocking()
        }));
    }

    fn render_update_status(&mut self, ui: &mut egui::Ui) {
        if let Some(promise) = &self.update_check {
            if promise.ready().is_none() {
                ui.spinner();
                ui.label("checking for updates");
                return;
            }
        }

        if let Some(promise) = self.update_check.take() {
            match promise.block_and_take() {
                Ok(Some(version)) => {
                    self.update_available = Some(version);
                    self.update_error = None;
                }
                Ok(None) => {
                    self.update_error = None;
                    self.update_available = None;
                }
                Err(e) => {
                    self.update_error = Some(e.to_string());
                    self.update_available = None;
                }
            }
        }
    }

    fn add_employee(&mut self) {
        let mandatory_fields = [
            (&self.employee_name, "Employee Name"),
            (&self.employee_position, "Employee Position"),
            (&self.pay_rate, "Pay Rate"),
        ];

        let mut missing_fields = Vec::new();
        for (field, field_name) in mandatory_fields.iter() {
            if field.is_empty() {
                missing_fields.push(*field_name);
            }
        }

        if !missing_fields.is_empty() {
            self.search_status = format!(
                "Please enter the following fields: {}",
                missing_fields.join(", ")
            );
            return;
        }

        if !self.employee_name.is_empty() && !self.employee_position.is_empty() {
            self.conn
                .execute(
                    "INSERT INTO employees (name, position, address, city, state, phone, filing_status, dependents, pay_rate) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![&self.employee_name, &self.employee_position, &self.address, &self.city, &self.state, &self.phone, &self.filing_status, &self.dependents, &self.pay_rate],
                )
                .expect("Failed to add employee");
            self.search_status = "Employee added successfully".to_string();
            self.employee_name.clear();
            self.employee_position.clear();
            self.address.clear();
            self.city.clear();
            self.state.clear();
            self.phone.clear();
            self.filing_status.clear();
            self.dependents.clear();
            self.pay_rate.clear();
        } else {
            self.search_status = "Please enter both name and position".to_string();
            println!("error adding employee");
        }
    }

    fn render_home(&mut self, ui: &mut egui::Ui) {
        ui.heading("Welcome to the Home Page");
        ui.heading("Stock Prices");

        let stock_service = stockservice::StockService::new();
        match stock_service.get_stock_data() {
            Ok((wba_data, cvs_data)) => {
                ui.horizontal(|ui| {
                    // Walgreens Stock
                    ui.vertical(|ui| {
                        let walgreens = &wba_data.quote;
                        ui.label(format!("Walgreens (WBA)"));
                        ui.label(format!("Price: ${:.2}", walgreens.current_price));
                        ui.colored_label(
                            if walgreens.change >= 0.0 {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            },
                            format!(
                                "Change: {:.2} ({:.2}%)",
                                walgreens.change, walgreens.change_percent
                            ),
                        );

                        //// Create chart using historical data
                        //let plot_points: PlotPoints = PlotPoints::new(
                        //    wba_data
                        //        .historical_data
                        //        .prices
                        //        .iter()
                        //        .enumerate()
                        //        .map(|(i, &price)| [i as f64, price])
                        //        .collect(),
                        //);
                        //
                        //Plot::new("WBA_chart").height(200.0).show(ui, |plot_ui| {
                        //    plot_ui.line(
                        //        egui_plot::Line::new(plot_points).name("Walgreens Stock Price"),
                        //    );
                        //});
                        //
                        // Display dates
                        ui.horizontal(|ui| {
                            for date in &wba_data.historical_data.dates {
                                ui.label(date);
                            }
                        });
                    });

                    ui.add_space(20.0);

                    // CVS Stock (similar implementation)
                    ui.vertical(|ui| {
                        let cvs = &cvs_data.quote;
                        ui.label(format!("CVS (CVS)"));
                        ui.label(format!("Price: ${:.2}", cvs.current_price));
                        ui.colored_label(
                            if cvs.change >= 0.0 {
                                egui::Color32::GREEN
                            } else {
                                egui::Color32::RED
                            },
                            format!("Change: {:.2} ({:.2}%)", cvs.change, cvs.change_percent),
                        );

                        //// Create chart using historical data
                        //let plot_points: PlotPoints = PlotPoints::new(
                        //    cvs_data
                        //        .historical_data
                        //        .prices
                        //        .iter()
                        //        .enumerate()
                        //        .map(|(i, &price)| [i as f64, price])
                        //        .collect(),
                        //);
                        //
                        //Plot::new("CVS_chart").height(200.0).show(ui, |plot_ui| {
                        //    plot_ui.line(egui_plot::Line::new(plot_points).name("CVS Stock Price"));
                        //});

                        // Display dates
                        ui.horizontal(|ui| {
                            for date in &cvs_data.historical_data.dates {
                                ui.label(date);
                            }
                        });
                    });
                });
            }
            Err(e) => {
                ui.label(format!("Error fetching stock data: {}", e));
            }
        }

        //additional content can go here
        ui.add_space(20.0);
        ui.heading("Welcome to the Medical Arts Pharmacy Payroll System");
        ui.heading("Manage your pharmacy's employee information and payroll with ease");
    }

    fn render_admin(&mut self, ui: &mut egui::Ui) {
        ui.heading("Admin Panel");
        ui.label("Manage administrative settings here.");
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Application Updates");
            if ui.button("Check for Updates").clicked() {
                self.check_for_update();
            }
        });

        if let Some(error) = &self.update_error {
            ui.colored_label(egui::Color32::RED, "Update Error: ");
            ui.label(error);
        }

        if let Some(version) = &self.update_available {
            ui.horizontal(|ui| {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("Update available: {}", version),
                );
                if ui.button("Download").clicked() {
                    perform_update();
                }
            });
        }
    }

    fn save_payroll(&mut self) {
        if let Some(employee) = &self.selected_employee {
            let _today = chrono::Local::now().format("%Y-%m-%d").to_string();
            for (i, &hours) in self.payroll_entries.iter().enumerate() {
                let date = chrono::Local::now()
                    .naive_local()
                    .date()
                    .checked_add_days(chrono::Days::new(i as u64))
                    .expect("Date calculation failed");
                let date_str = date.format("%Y-%m-%d").to_string();

                self.conn
                    .execute(
                        "INSERT INTO payroll (employee_id, date, hours) VALUES(?1, ?2, ?3)",
                        rusqlite::params![employee.id, date_str, hours],
                    )
                    .expect("Failed to save payroll");
            }
            println!("Payroll saved for {}!", employee.name);
        }
    }
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
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_update_status(ui);
            });
        });

        // Dynamically change the entire CentralPanel based on the selected button
        egui::CentralPanel::default().show(ctx, |ui| match self.active_panel {
            ActivePanel::Home => self.render_home(ui),
            ActivePanel::Admin => self.render_admin(ui),
            ActivePanel::Payroll => render_payroll(self, ui),
            ActivePanel::Employees => render_employees(self, ui),
        });
    }
}
