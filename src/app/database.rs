use crate::app::employee::Employee;
use crate::app::payroll::PayrollEntry;
use crate::app::app::PharmacyApp;
use rusqlite::Connection;
use std::path::PathBuf;
use rusqlite::params;

pub fn search_employee(app: &mut PharmacyApp) {
    let search_pattern = format!("%{}%", app.search_name);
    let sql = "SELECT * FROM employees WHERE name LIKE ?1";
    let mut stmt = match app.conn.prepare(sql) {
        Ok(stmt) => stmt,
        Err(e) => {
            app.search_status = format!("Error: {}", e);
            return;
        }
    };

    let mut rows = match stmt.query_map(params![search_pattern], |row| {
        Ok(Employee {
            id: row.get(0)?,
            name: row.get(1)?,
            position: row.get(2)?,
            address: row.get(3)?,
            city: row.get(4)?,
            state: row.get(5)?,
            phone: row.get(6)?,
            filing_status: row.get(7)?,
            dependents: row.get(8)?,
            pay_rate: row.get(9)?,
        })
    }) {
        Ok(rows) => rows,
        Err(e) => {
            app.search_status = format!("Error: {}", e);
            return;
        }
    };

    app.search_result = rows.next().and_then(Result::ok);
    app.search_status = if app.search_result.is_some() {
        println!("found employee");
        "Employee Found".to_string()
    } else {
        println!("no employee found");
        "No employee found".to_string()
    };
}

pub fn add_employee(app: &mut PharmacyApp) {
    let mandatory_fields = [
        (&app.employee_name, "Employee Name"),
        (&app.employee_position, "Employee Position"),
        (&app.pay_rate, "Pay Rate"),
    ];

    let mut missing_fields = Vec::new();
    for (field, field_name) in mandatory_fields.iter() {
        if field.is_empty() {
            missing_fields.push(*field_name);
        }
    }

    if !missing_fields.is_empty() {
        app.search_status = format!(
            "Please enter the following fields: {}",
            missing_fields.join(", ")
        );
        return;
    }

    if !app.employee_name.is_empty() && !app.employee_position.is_empty() {
        app.conn
            .execute(
                "INSERT INTO employees (
                        name,
                        position,
                        address,
                        city,
                        state,
                        phone,
                        filing_status,
                        dependents,
                        pay_rate)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    &app.employee_name,
                    &app.employee_position,
                    &app.address,
                    &app.city,
                    &app.state,
                    &app.phone,
                    &app.filing_status,
                    &app.dependents,
                    &app.pay_rate
                ],
            )
            .expect("Failed to add employee");
        app.search_status = "Employee added successfully".to_string();
        app.employee_name.clear();
        app.employee_position.clear();
        app.address.clear();
        app.city.clear();
        app.state.clear();
        app.phone.clear();
        app.filing_status.clear();
        app.dependents.clear();
        app.pay_rate.clear();
    } else {
        app.search_status = "Please enter both name and position".to_string();
        println!("error adding employee");
    }
}
pub fn get_all_employees(conn: &Connection) -> Result<Vec<Employee>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM employees")?;
    let employees: Vec<Employee> = stmt
        .query_map([], |row| {
            Ok(Employee {
                id: row.get(0)?,
                name: row.get(1)?,
                position: row.get(2)?,
                address: row.get(3)?,
                city: row.get(4)?,
                state: row.get(5)?,
                phone: row.get(6)?,
                filing_status: row.get(7)?,
                dependents: row.get(8)?,
                pay_rate: row.get(9)?,
            })
        })?
        .filter_map(Result::ok)
        .collect();
    Ok(employees)
}

pub fn initialize_tables(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS employees (
            id INTEGER PRIMARY KEY, 
            name TEXT, 
            position TEXT, 
            address TEXT, 
            city TEXT, 
            state TEXT, 
            phone TEXT, 
            filing_status TEXT, 
            dependents TEXT, 
            pay_rate TEXT)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS payroll (
            id INTEGER PRIMARY KEY,
            employee_id INTEGER,
            hours_worked REAL,
            date_of_pay TEXT,
            gross REAL,
            withholding REAL,
            social_security REAL,   
            net REAL,
            roth_ira REAL)",
        [],
    )?;

    Ok(())
}

pub fn get_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("my_payroll_app");
    std::fs::create_dir_all(&path).expect("Failed to create data directory");
    path.push("employees.db");
    path
}

pub fn delete_payroll_entry(conn: &Connection, id: i64) -> Result<(), rusqlite::Error> {
    conn.execute("DELETE FROM payroll WHERE id = ?", [id])?;
    Ok(())
}

pub fn get_all_payroll_entries(conn: &Connection) -> Result<Vec<PayrollEntry>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM payroll")?;
    let payroll_entries: Vec<PayrollEntry> = stmt
        .query_map([], |row| {
            Ok(PayrollEntry {
                id: row.get(0)?,
                employee_id: row.get(1)?,
                hours_worked: row.get(2)?,
                date_of_pay: row.get(3)?,
                gross: row.get(4)?,
                withholding: row.get(5)?,
                social_security: row.get(6)?,
                net: row.get(7)?,
                roth_ira: row.get(8)?,
            })
        })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(payroll_entries)
}
