use super::stockservice;

pub fn render_home(ui: &mut egui::Ui) {
    ui.heading("Welcome to the Home Page");
    ui.heading("Stock Prices");

    let stock_service = stockservice::StockService::instance();
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
