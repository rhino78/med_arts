use super::database;
use crate::app::app::get_fridays_of_year;
use crate::app::app::PharmacyApp;
use crate::app::database::add_employee;
use crate::app::database::search_employee;
use crate::app::payroll::PayrollEntry;
use egui::Ui;

pub fn render_payroll(app: &mut PharmacyApp, ui: &mut Ui) {
    let fridays = get_fridays_of_year();

    let employees = match database::get_all_employees(&app.conn) {
        Ok(employees) => employees,
        Err(e) => {
            ui.label(format!("Error fetching employees {}", e));
            return;
        }
    };

    let mut selected_employee_index = 0;
    let employee_names: Vec<String> = employees.iter().map(|e| e.name.clone()).collect();

    ui.vertical(|ui| {
        ui.label("Select Employee");
        egui::ComboBox::from_id_source("employee_select")
            .selected_text(if employee_names.is_empty() {
                "No Employees found".to_string()
            } else {
                employee_names[selected_employee_index].clone()
            })
            .show_ui(ui, |ui| {
                for (index, name) in employee_names.iter().enumerate() {
                    ui.selectable_value(&mut selected_employee_index, index, name);
                }
            });
        if employee_names.is_empty() {
            return;
        }
    });

    let selected_employee = &employees[selected_employee_index];
    let pay_rate: f32 = selected_employee.pay_rate.parse().unwrap_or(0.0);

    //calculate the values
    let gross = app.calculate_gross(app.hours_worked, pay_rate);
    let withholding = app.calculate_withholding(gross);
    let social_security = app.calculate_social_security(gross);
    let net = app.calculate_net(gross, withholding, social_security);

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Date of Pay:");
                egui::ComboBox::from_id_source("date_select")
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

    if ui.button("Save").clicked() {
        let _entry = PayrollEntry {
            date_of_pay: app.selected_friday.clone(),
            gross,
            net,
            employee_id: selected_employee_index as i32,
            hours_worked: app.hours_worked,
            withholding,
            roth_ira: app.roth_ira,
            social_security,
            id: 0,
        };
        let res = _entry.save_to_db(&app.conn);
        println!("resut: {:?}", res);
    }
    ui.add_space(20.0);
    ui.separator();
    ui.label("Payroll History");
    let payroll_entries = match database::get_all_payroll_entries(&app.conn) {
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
                        ui.label(&entry.date_of_pay);
                        //ui.label(&entry.employee_id);
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
                        }
                        ui.end_row();
                    }
                });
        });
    }
}

pub fn render_employees(app: &mut PharmacyApp, ui: &mut Ui) {
    ui.heading("Employees Panel");

    let employees = match database::get_all_employees(&app.conn) {
        Ok(e) => e,
        Err(e) => {
            ui.label(format!("Error: {}", e));
            return;
        }
    };

    ui.horizontal(|ui| {
        ui.label("Select Employee");
        egui::ComboBox::from_label("")
            .selected_text(
                app.selected_employee
                    .as_ref()
                    .map(|emp| emp.name.clone())
                    .unwrap_or_else(|| "Select an employee".to_string()),
            )
            .show_ui(ui, |ui| {
                for emp in &employees {
                    ui.selectable_value(&mut app.selected_employee_id, Some(emp.id), &emp.name);
                }
            });
        if ui.button("Add New Employee").clicked() {
            app.show_add_employee_popup = true;
        }
    });

    if let Some(selected_id) = app.selected_employee_id {
        if app
            .selected_employee
            .as_ref()
            .map_or(true, |emp| emp.id != selected_id)
        {
            match database::get_employee_by_id(&app.conn, selected_id) {
                Ok(employee) => {
                    app.selected_employee = Some(employee);
                }
                Err(e) => {
                    ui.label(format!("Error fetching Employee {}", e));
                }
            }
        }
    }

    if let Some(employee) = &app.selected_employee {
        ui.separator();
        ui.heading("Employee Details");

        ui.vertical(|ui| {
            ui.label(format!("ID: {}", employee.id));
            ui.label(format!("Name: {}", employee.name));
            ui.label(format!("Position: {}", employee.position));
            ui.label(format!("Address: {}", employee.address));
            ui.label(format!("City: {}", employee.city));
            ui.label(format!("State: {}", employee.state));
            ui.label(format!("Phone: {}", employee.phone));
            ui.label(format!("Filiing Status: {}", employee.filing_status));
            ui.label(format!("Dependendts: {}", employee.dependents));
        });
    }

    if app.show_add_employee_popup {
        egui::Window::new("Add New Employee")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ui.ctx(), |ui| {
                // Use columns for better alignment
                ui.columns(2, |columns| {
                    for (i, column) in columns.iter_mut().enumerate() {
                        column.vertical(|ui| match i {
                            0 => {
                                ui.label("Name:").on_hover_text("Enter employee's name");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.employee_name)
                                            .hint_text("John Doe"),
                                    );
                                });
                                ui.add_space(1.0);

                                ui.label("Address:")
                                    .on_hover_text("Enter employee's address");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.address)
                                            .hint_text("123 Main St"),
                                    );
                                });
                                ui.add_space(1.0);

                                ui.label("State:").on_hover_text("Enter state");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.state).hint_text("TX"),
                                    );
                                });
                                ui.add_space(1.0);

                                ui.label("Filing Status:")
                                    .on_hover_text("Enter tax filing status");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.filing_status)
                                            .hint_text("Single"),
                                    );
                                });
                            }
                            1 => {
                                ui.label("Position:").on_hover_text("Enter the job title");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.employee_position)
                                            .hint_text("Software Engineer"),
                                    );
                                });
                                ui.add_space(1.0);

                                ui.label("City:").on_hover_text("Enter city");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.city)
                                            .hint_text("Austin"),
                                    );
                                });
                                ui.add_space(1.0);

                                ui.label("Pay Rate:").on_hover_text("Enter Rate of Pay ");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.pay_rate)
                                            .hint_text("0"),
                                    );
                                });

                                ui.add_space(1.0);
                                ui.label("Phone:").on_hover_text("Enter phone number");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.phone)
                                            .hint_text("(512) 555-1234"),
                                    );
                                });
                                ui.add_space(1.0);

                                ui.label("Dependents:")
                                    .on_hover_text("Enter number of dependents");
                                egui::Frame::default().show(ui, |ui| {
                                    ui.add_sized(
                                        [150.0, 25.0],
                                        egui::TextEdit::singleline(&mut app.dependents)
                                            .hint_text("0"),
                                    );
                                });
                            }
                            _ => {}
                        });
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        add_employee(app);
                        app.show_add_employee_popup = false;
                    }
                    if ui.button("Cancel").clicked() {
                        add_employee(app);
                        app.show_add_employee_popup = false;
                    }
                });
            });
    }
}
