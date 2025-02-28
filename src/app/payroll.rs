use rusqlite::params;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PayrollEntry {
    pub id: i64,
    pub date_of_pay: String,
    pub employee_id: String,
    pub hours_worked: f32,
    pub gross: f32,
    pub withholding: f32,
    pub roth_ira: f32,
    pub social_security: f32,
    pub net: f32,
}

impl PayrollEntry {
    pub fn save_to_db(&self, conn: &Connection) -> Result<i64, rusqlite::Error> {
        println!("Saving to DB: {}", self.date_of_pay);
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
                &self.employee_id,    
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
