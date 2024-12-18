use crate::io::util::print_table;
use crate::io::writer::write_table_definition;
use crate::types::types::{InsertDefinition, SelectDefinition, TableDefinition, UpdateDefinition};
use std::io;
use std::io::Write;

pub struct QueryRunner;

impl QueryRunner {
    pub fn run_command(command: &str) -> io::Result<()> {
        let command = command.trim();
        if command.starts_with("CREATE TABLE") {
            if let Some(table_def) = TableDefinition::from_sql(command) {
                write_table_definition(&table_def)?;
                println!("Table '{}' created successfully.", table_def.name);
            } else {
                println!("Error: Invalid CREATE TABLE syntax.");
            }
        } else if command.starts_with("INSERT INTO") {
            if let Some(mut insert_def) = InsertDefinition::from_sql(command) {
                insert_def.validate_and_insert()?;
                println!("Row inserted successfully into table '{}'.", insert_def.name);
            } else {
                println!("Error: Invalid INSERT INTO syntax.");
            }
        } else if command.starts_with("SELECT") {
            if let Some(select_def) = SelectDefinition::from_sql(command) {
                let rows = select_def.execute()?;
                print_table(rows);
            }
        } else if command.starts_with("UPDATE") {
            if let Some(update_def) = UpdateDefinition::from_sql(command) {
                match update_def.load_data() {
                    Ok(column_map) => {

                    }
                    Err(err) => println!("Error loading data: {}", err),
                }
            } else {
                println!("Error: Invalid UPDATE syntax.");
            }
        } else {
            println!("Error: Unsupported command.");
        }
        Ok(())
    }

    pub fn repl() -> io::Result<()> {
        println!("Welcome to the RQL. Type your RQL commands below. Type 'EXIT' to quit.");

        loop {
            print!("rql> ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            let input = input.trim();
            if input.eq_ignore_ascii_case("EXIT") {
                println!("Exiting REPL.");
                break;
            }

            if let Err(e) = QueryRunner::run_command(input) {
                eprintln!("Error: {}", e);
            }
        }

        Ok(())
    }
}