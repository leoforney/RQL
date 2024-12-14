use crate::io::reader::{read_table_definition, read_vec_of_bytes_from_file};
use crate::io::writer::{append_vec_of_bytes_to_file, write_table_definition};
use crate::types::types::{ColumnDefinition, DataType, InsertDefinition, SelectDefinition, TableDefinition, Value};
use std::collections::HashMap;
use std::io;
use std::io::Write;
use prettytable::{Cell, Row, Table};
use prettytable::format;

impl DataType {
    pub(crate) fn from_sql_type(sql_type: &str) -> Option<DataType> {
        match sql_type.to_uppercase().as_str() {
            "INTEGER" => Some(DataType::Integer),
            "FLOAT" => Some(DataType::Float),
            "TEXT" => Some(DataType::Text),
            "BOOLEAN" => Some(DataType::Boolean),
            _ => None,
        }
    }
    pub(crate) fn to_sql_type(&self) -> &str {
        match self {
            DataType::Integer => "INTEGER",
            DataType::Float => "FLOAT",
            DataType::Text => "TEXT",
            DataType::Boolean => "BOOLEAN",
        }
    }

    fn rust_type(&self) -> &str {
        match self {
            DataType::Integer => "i32",
            DataType::Float => "f64",
            DataType::Text => "String",
            DataType::Boolean => "bool",
        }
    }
}

impl TableDefinition {
    pub(crate) fn from_sql(sql: &str) -> Option<TableDefinition> {
        let sql = sql.trim();
        if !sql.starts_with("CREATE TABLE") || !sql.ends_with(");") {
            return None;
        }

        let name_start = "CREATE TABLE ".len();
        let name_end = sql.find('(')?;
        let name = sql[name_start..name_end].trim().to_string();

        let columns_start = name_end + 1;
        let columns_end = sql.len() - 2;
        let columns_str = sql[columns_start..columns_end].trim();

        let columns: Vec<ColumnDefinition> = columns_str
            .split(',')
            .filter_map(|col| ColumnDefinition::from_sql(col.trim()))
            .collect();

        if columns.is_empty() {
            return None;
        }

        Some(TableDefinition { name, columns })
    }
    pub(crate) fn to_sql(&self) -> String {
        let column_definitions: Vec<String> = self
            .columns
            .iter()
            .map(|col| col.to_sql())
            .collect();

        format!(
            "CREATE TABLE {} (\n  {}\n);",
            self.name,
            column_definitions.join(",\n  ")
        )
    }
}

impl ColumnDefinition {
    pub(crate) fn from_sql(sql: &str) -> Option<ColumnDefinition> {
        let parts: Vec<&str> = sql.split_whitespace().collect();

        if parts.len() < 2 {
            return None;
        }

        let name = parts[0].to_string();
        let data_type = DataType::from_sql_type(parts[1])?;

        let nullable = !sql.contains("NOT NULL");
        let unique = sql.contains("UNIQUE");

        Some(ColumnDefinition {
            name,
            data_type,
            nullable,
            unique,
        })
    }
    pub(crate) fn to_sql(&self) -> String {
        format!(
            "{} {}{}{}",
            self.name,
            self.data_type.to_sql_type(),
            if self.nullable { "" } else { " NOT NULL" },
            if self.unique { " UNIQUE" } else { "" }
        )
    }
}

impl InsertDefinition {
    pub fn from_sql(sql: &str) -> Option<Self> {
        let sql = sql.trim();
        if !sql.starts_with("INSERT") || !sql.contains("VALUES") {
            return None;
        }

        let name_start = "INSERT INTO ".len();
        let name_end = sql.find("VALUES")?;
        let table_name = sql[name_start..name_end].trim().to_string();

        let values_start = sql.find('(')? + 1;
        let values_end = sql.rfind(')')?;
        let values_str = sql[values_start..values_end].trim();

        let values: Vec<String> = values_str.split(',').map(|v| v.trim().to_string()).collect();

        Some(InsertDefinition {
            name: table_name,
            table_definition: TableDefinition {
                name: String::new(),
                columns: Vec::new(),
            },
            values,
        })
    }

    pub fn validate_and_insert(&mut self) -> io::Result<()> {
        let table_definition = read_table_definition(&self.name)?;
        self.table_definition = table_definition;

        if self.values.len() != self.table_definition.columns.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Number of values does not match the table definition",
            ));
        }

        let mut row_data = Vec::new();
        for (value, column) in self.values.iter().zip(self.table_definition.columns.iter()) {
            let serialized_value = match column.data_type {
                DataType::Integer => {
                    let parsed: i32 = value.parse().map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "Invalid integer value")
                    })?;
                    bincode::serialize(&parsed).unwrap()
                }
                DataType::Float => {
                    let parsed: f64 = value.parse().map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "Invalid float value")
                    })?;
                    bincode::serialize(&parsed).unwrap()
                }
                DataType::Text => bincode::serialize(&value).unwrap(),
                DataType::Boolean => {
                    let parsed: bool = value.parse().map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "Invalid boolean value")
                    })?;
                    bincode::serialize(&parsed).unwrap()
                }
            };
            row_data.extend(serialized_value);
        }

        append_vec_of_bytes_to_file(vec![vec![row_data]], self.name.as_str())?;

        Ok(())
    }
}

impl SelectDefinition {
    pub fn from_sql(sql: &str) -> Option<Self> {
        let sql = sql.trim().trim_end_matches(';'); // Remove trailing semicolon
        if !sql.starts_with("SELECT") || !sql.contains("FROM") {
            return None;
        }

        let table_start = sql.find("FROM")? + 5;
        let table_end = sql.find("WHERE").unwrap_or_else(|| sql.len());
        let table_name = sql[table_start..table_end].trim().to_string();

        let criteria = if let Some(where_index) = sql.find("WHERE") {
            sql[where_index + 6..].trim().to_string()
        } else {
            String::new()
        };

        let parsed_criteria = SelectDefinition::parse_criteria(&criteria)?;

        Some(SelectDefinition {
            table_name,
            criteria: parsed_criteria,
        })
    }

    pub fn execute(&self) -> io::Result<Vec<HashMap<String, Value>>> {
        let all_rows = read_vec_of_bytes_from_file(self.table_name.as_str())?;

        let filtered_rows: Vec<HashMap<String, Value>> = all_rows
            .into_iter()
            .filter(|row| self.matches_criteria(row))
            .collect();

        Ok(filtered_rows)
    }

    fn parse_criteria(criteria: &str) -> Option<Vec<(String, String, String)>> {
        if criteria.is_empty() {
            return Some(vec![]);
        }

        let conditions: Vec<&str> = criteria
            .split("AND")
            .flat_map(|part| part.split("OR"))
            .map(|s| s.trim())
            .collect();

        let mut parsed_conditions = vec![];

        for condition in conditions {
            let parts: Vec<&str> = condition.split('=').collect();
            if parts.len() != 2 {
                return None;
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().replace("'", "").to_string();

            parsed_conditions.push((key, "=".to_string(), value)); // TODO: Implement more than =
        }

        Some(parsed_conditions)
    }

    fn matches_criteria(&self, row: &HashMap<String, Value>) -> bool {
        self.criteria.iter().all(|(key, operator, value)| {
            match operator.as_str() {
                "=" => row.get(key).map(|v| v.to_string() == *value).unwrap_or(false),
                _ => false,
            }
        })
    }
}

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

                let mut table = Table::new();
                table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
                
                let mut column_order: Vec<String> = Vec::new(); // TODO: Match order with table definition and allow select of individual/grouped columns
                
                if let Some(first_row) = rows.get(0) {
                    column_order = first_row.keys().cloned().collect();
                    let headers: Vec<Cell> = column_order.iter().map(|key| Cell::new(key)).collect();
                    table.set_titles(Row::new(headers));
                }
                
                for row in rows {
                    let cells: Vec<Cell> = column_order
                        .iter()
                        .map(|key| {
                            row.get(key)
                                .map(|value| Cell::new(&*value.to_string()))
                                .unwrap_or_else(|| Cell::new(""))
                        })
                        .collect();
                    table.add_row(Row::new(cells));
                }
                
                table.printstd();
            }
        }
        else {
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