use crate::app::app::get_available_fridays;
use crate::app::app::get_fridays_of_year;
use crate::app::app::PharmacyApp;
use crate::app::database;
use egui::Ui;
use rusqlite::params;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollEntry {
    pub id: i64,
    pub date_of_pay: String,
    pub employee_id: i32,
    pub hours_worked: f32,
    pub gross: f32,
    pub withholding: f32,
    pub roth_ira: f32,
    pub social_security: f32,
    pub net: f32,
}

impl PayrollEntry {
    pub fn save_to_db(&self, conn: &Connection) -> Result<i64, rusqlite::Error> {
        match conn.execute(
            "INSERT INTO payroll (
                date_of_pay,
                employee_id,
                hours_worked,
                gross,
                withholding,
                social_security,
                roth_ira,
                net) 
            VALUES (
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                ?7,
                ?8)",
            params![
                &self.date_of_pay,
                &self.employee_id + 1,
                &self.hours_worked,
                self.gross,
                self.withholding,
                self.social_security,
                self.roth_ira,
                self.net
            ],
        ) {
            Ok(_) => Ok(conn.last_insert_rowid()),
            Err(e) => Err(e),
        }
    }
}

pub fn render_payroll(app: &mut PharmacyApp, ui: &mut Ui) {
    let fridays = get_fridays_of_year();

    let employees = match database::get_all_employees(&app.conn) {
        Ok(employees) => employees,
        Err(e) => {
            ui.label(format!("Error fetching employees {}", e));
            return;
        }
    };

    let employee_names: Vec<String> = employees.iter().map(|e| e.name.clone()).collect();

    ui.vertical(|ui| {
        ui.label("Select Employee");
        egui::ComboBox::from_id_salt("employee_select")
            .selected_text(if employee_names.is_empty() {
                "No Employees found".to_string()
            } else {
                employee_names[app.selected_employee_index].clone()
            })
            .show_ui(ui, |ui| {
                for (index, name) in employee_names.iter().enumerate() {
                    if ui
                        .selectable_value(&mut app.selected_employee_index, index, name)
                        .clicked()
                    {
                        app.refresh_available_fridays();
                    }
                }
            });
        if employee_names.is_empty() {
            app.refresh_available_fridays();
        }
    });

    let selected_employee = &employees[app.selected_employee_index];
    let pay_rate: f32 = selected_employee.pay_rate.parse().unwrap_or(0.0);
    let available_fridays = get_available_fridays(&app.conn, selected_employee.id);
    if !available_fridays.contains(&app.selected_friday) && !available_fridays.is_empty() {
        app.selected_friday = available_fridays[0].clone();
    }

    //calculate the values
    let gross = calculate_gross(app.hours_worked, pay_rate);
    let withholding = calculate_withholding(gross);
    let social_security = calculate_social_security(gross);
    let net = calculate_net(gross, withholding, social_security);

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Date of Pay:");
                egui::ComboBox::from_id_salt("date_select")
                    .selected_text(app.selected_friday.clone())
                    .show_ui(ui, |ui| {
                        for friday in &fridays {
                            if ui
                                .selectable_value(
                                    &mut app.selected_friday,
                                    friday.clone(),
                                    friday.clone(),
                                )
                                .clicked()
                            {
                                println!("Selected Friday: {}", friday);
                            }
                        }
                    })
            });

            ui.vertical(|ui| {
                ui.label("Hours");
                ui.add(egui::DragValue::new(&mut app.hours_worked).speed(0.1));
            });

            ui.vertical(|ui| {
                ui.label("Gross");
                ui.add(egui::Label::new(format!("{:.2}", gross)));
            });
            ui.vertical(|ui| {
                ui.label("Withholding");
                ui.add(egui::Label::new(format!("{:.2}", withholding)));
            });

            ui.vertical(|ui| {
                ui.label("Social Security");
                ui.add(egui::Label::new(format!("{:.2}", social_security)));
            });

            ui.vertical(|ui| {
                ui.label("Roth IRA");
                ui.add(egui::DragValue::new(&mut app.roth_ira).speed(0.1));
            });

            ui.vertical(|ui| {
                ui.label("Net");
                ui.add(egui::Label::new(format!("{:.2}", net)));
            });
        });
    });

    let save_button = ui.add_enabled(!available_fridays.is_empty(), egui::Button::new("Save"));
    if save_button.clicked() {
        let _entry = PayrollEntry {
            date_of_pay: app.selected_friday.clone(),
            gross,
            net,
            employee_id: app.selected_employee_index as i32,
            hours_worked: app.hours_worked,
            withholding,
            roth_ira: app.roth_ira,
            social_security,
            id: 0,
        };

        let _res = _entry.save_to_db(&app.conn);
        app.refresh_available_fridays();
    }
    ui.add_space(20.0);
    ui.separator();
    ui.label("Payroll History");
    let payroll_entries = match database::get_payroll_by_id(&app.conn, selected_employee.id) {
        Ok(e) => e,
        Err(e) => {
            ui.label(format!("Error: {}", e));
            return;
        }
    };

    if payroll_entries.is_empty() {
        ui.label("No payroll entries found");
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("payroll_entries_grid")
                .striped(true)
                .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    ui.strong("Date of Pay");
                    ui.strong("Gross");
                    ui.strong("Net");
                    //ui.strong("Employee Name");
                    ui.strong("Hours Worked");
                    ui.strong("Withholding");
                    ui.strong("Roth IRA");
                    ui.strong("Social Security");
                    ui.end_row();
                    for entry in &payroll_entries {
                        ui.label(format!("{}", entry.date_of_pay));
                        ui.label(format!("{:.2}", entry.gross));
                        ui.label(format!("{:.2}", entry.net));
                        ui.label(format!("{}", entry.hours_worked));
                        ui.label(format!("{:.2}", entry.withholding));
                        ui.label(format!("{:.2}", entry.roth_ira));
                        ui.label(format!("{:.2}", entry.social_security));

                        if ui.button("Delete").clicked() {
                            database::delete_payroll_entry(&app.conn, entry.id).unwrap_or_else(
                                |e| println!("Error deleting payroll entry: {}", e),
                            );
                            app.refresh_available_fridays();
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

pub fn calculate_gross(hours_worked: f32, pay_rate: f32) -> f32 {
    hours_worked * pay_rate
}

pub fn calculate_withholding(gross: f32) -> f32 {
    gross * 0.2
}

pub fn calculate_social_security(gross: f32) -> f32 {
    gross * 0.075
}

pub fn calculate_net(gross: f32, withholding: f32, social_security: f32) -> f32 {
    gross - withholding - social_security
}
