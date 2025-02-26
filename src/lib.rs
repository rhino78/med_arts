pub mod app;

#[cfg(test)]
mod tests {
    use crate::app::app::ActivePanel;
    use crate::app::app::PharmacyApp;
    use crate::app::database::get_all_employees;
    use crate::app::employee::Employee;
    use rusqlite::Connection;
    use rusqlite::Result;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    fn setup_test_db() -> Result<Connection> {
        let conn = Connection::open_in_memory().expect("Failed to create test database");
        conn.execute(
            "CREATE TABLE 
            employees (id INTEGER PRIMARY KEY, name TEXT, position TEXT, address TEXT, city TEXT, state TEXT, phone TEXT, filing_status TEXT, dependents TEXT, pay_rate TEXT)",[],)?;

        conn.execute("CREATE TABLE 
            payroll (id INTEGER PRIMARY KEY, employee_id INTEGER, date TEXT, hours REAL, FOREIGN KEY(employee_id) REFERENCES employees(id))",[],)?;

        conn.execute(
            "INSERT INTO employees (name, position, address, city, state, phone, filing_status, dependents, pay_rate) 
            VALUES ('Bob', 'Manager', '456 Main St', 'Anytown', 'CA', '987-654-3210', 'single', '0', '50')",
            [],
        )?;

        conn.execute(
            "INSERT INTO employees (name, position, address, city, state, phone, filing_status, dependents, pay_rate) 
            VALUES ('Ryan', 'Manager', '456 Main St', 'Anytown', 'CA', '987-654-3210', 'single', '0', '50')",[])?;

        Ok(conn)
    }

    #[test]
    fn test_calculate_net() {
        let app = PharmacyApp::new();
        let gross = 1000.0;
        let withholding = 100.0;
        let social_security = 75.0;
        let expected_net = 825.0;
        assert_eq!(
            app.calculate_net(gross, withholding, social_security),
            expected_net
        );
    }

    #[test]
    fn test_calculate_social_security() {
        let app = PharmacyApp::new();
        let gross = 1000.0;
        let expected_social_security = 75.0;
        assert_eq!(
            app.calculate_social_security(gross),
            expected_social_security
        );
    }

    #[test]
    fn test_calculate_withholding() {
        let app = PharmacyApp::new();
        let gross = 1000.0;
        let expected_withholding = 200.0;
        assert_eq!(app.calculate_withholding(gross), expected_withholding);
    }

    #[test]
    fn test_calculate_gross() {
        let app = PharmacyApp::new();
        let hours_worked = 40.0;
        let pay_rate = 20.0;
        let expected_gross = 800.0;
        assert_eq!(app.calculate_gross(hours_worked, pay_rate), expected_gross);
    }

    #[test]
    fn test_add_payroll_entry() {
        let conn = setup_test_db().expect("Failed to create test database");

        conn.execute(
            "INSERT INTO payroll (employee_id, date, hours) VALUES (1, '2023-08-01', 8.0)",
            [],
        )
        .expect("Failed to add payroll entry");

        let mut stmt = conn
            .prepare("SELECT hours from payroll WHERE employee_id = 1")
            .unwrap();
        let hours: f64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(hours, 8.0, "Payroll entry should have been added");
    }

    #[test]
    fn test_insert_duplicate_employee() {
        let conn = setup_test_db().expect("Failed to create test database");

        // Attempt to insert a duplicate employee
        let result = conn.execute(
            "INSERT INTO employees (name, position, address, city,state, phone, filing_status, dependents, pay_rate)
            VALUES ('Ryan', 'Manager', '456 Main St', 'Anytown', 'CA', '987-654-3210', 'single', '0', '50')",
            [],
        );
        assert!(
            result.is_ok(),
            "Should allow inserting duplicate names unless restricted"
        );
    }

    #[test]
    fn test_update_employee() {
        let conn = setup_test_db().expect("Failed to create test database");

        conn.execute(
            "UPDATE employees SET position = 'Senior Manager' WHERE name = 'Ryan'",
            [],
        )
        .expect("Failed to update employee");

        let mut stmt = conn
            .prepare("SELECT position FROM employees WHERE name = 'Ryan'")
            .unwrap();
        let updated_position: String = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(updated_position, "Senior Manager");
    }

    #[test]
    fn test_delete_employee() {
        let conn = setup_test_db().expect("Failed to create test database");
        conn.execute("DELETE FROM employees WHERE name = 'Bob'", [])
            .expect("Failed to delete employee");

        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM employees WHERE name = 'Bob'")
            .unwrap();
        let count: u32 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 0, "Employee 'Bob' should have been deleted");
    }

    #[test]
    fn test_select_name() {
        let conn = setup_test_db().expect("Failed to create test database");
        let select_string = "select * from employees WHERE name like 'Ryan'";
        let mut stmt = conn.prepare(select_string).unwrap();
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
            })
            .unwrap()
            .filter_map(Result::ok)
            .collect();

        assert_eq!(
            employees.len(),
            1,
            "Expected 1 employee with name 'Ryan' in the database"
        );
    }

    #[test]
    fn test_get_all_employees() {
        let conn = setup_test_db().expect("Failed to create test database");
        // Create an instance of PharmacyApp with the test database
        let app = PharmacyApp {
            search_result: None,
            active_panel: ActivePanel::Home,
            admin_text: String::new(),
            employee_name: String::new(),
            search_name: String::new(),
            search_status: String::new(),
            employee_position: String::new(),
            conn,
            employees: vec![],
            selected_employee: None,
            payroll_entries: [0.0; 7],
            address: String::new(),
            city: String::new(),
            state: String::new(),
            phone: String::new(),
            filing_status: String::new(),
            dependents: String::new(),
            hours_worked: 0.0,
            withholding: 0.0,
            roth_ira: 0.0,
            social_security: 0.0,
            selected_friday: String::new(),
            net: 0.0,
            gross: 0.0,
            pay_rate: String::new(),
        };

        // Call get_all_employees()
        let employees = app.get_all_employees().expect("Failed to fetch employees");

        // Assertions
        assert_eq!(employees.len(), 2, "Expected 2 employees in the database");
        assert_eq!(employees[0].name, "Bob");
        assert_eq!(employees[0].position, "Manager");
        assert_eq!(employees[1].name, "Ryan");
        assert_eq!(employees[1].position, "Manager");

        for emp in &employees {
            println!("Name: {}, Position: {}", emp.name, emp.position);
        }
    }
}
