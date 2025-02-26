use chrono::NaiveDate;
use rusqlite::Connection;
use rusqlite::params;   
use serde::{Deserialize, Serialize};
use chrono::Datelike;

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollEntry {
    pub employee_name: String,
    pub hours_worked: f32,
    pub date_of_pay: String,
    pub gross: f32,
    pub withholding: f32,
    pub roth_ira: f32,
    pub social_security: f32,
    pub net: f32,
}

impl PayrollEntry {
    pub fn save_to_db(&self, conn: &Connection) {
        if let Ok(parsed_date) = NaiveDate::parse_from_str(&self.date_of_pay, "%Y-%m-%d") {
            let _ = conn.execute(
                "INSERT INTO payroll (employee_name, hours_worked, pay_date, gross, withholding, roth_ira, social_security, net) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    &self.employee_name,
                    &self.hours_worked,
                    &parsed_date.to_string(),
                    self.gross,
                    self.withholding,
                    self.roth_ira,
                    self.social_security,
                    self.net
                ],
            ).unwrap();
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
}
