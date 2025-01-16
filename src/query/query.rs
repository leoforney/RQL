use crate::io::reader::{read_table_definition, read_vec_of_bytes_from_file};
use crate::io::util::{print_table, reconstruct_rows};
use crate::io::writer::{serialize_from_value, serialize_value, write_vec_of_bytes_to_file};
use crate::rqle::rqle_parser::ExpressionParser;
use crate::rqle::shader_executor::ShaderExecutor;
use crate::types::types::{ColumnDefinition, DataType, InsertDefinition, SelectDefinition, TableDefinition, UpdateDefinition, Value};
use regex::Regex;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io;

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
            DataType::Float => "f32",
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
            let serialized_value = serialize_value(value, &column.data_type)?;
            row_data.extend(serialized_value);
        }

        write_vec_of_bytes_to_file(vec![vec![row_data]], self.name.as_str(), true)?;

        Ok(())
    }
}

impl SelectDefinition {
    pub fn from_sql(sql: &str) -> Option<Self> {
        let sql = sql.trim().trim_end_matches(';');
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

impl UpdateDefinition {
    pub fn from_sql(sql: &str) -> Option<Self> {
        let sql = sql.trim().trim_end_matches(';');

        if !sql.starts_with("UPDATE") || !sql.contains("SET") {
            return None;
        }

        let table_start = "UPDATE ".len();
        let table_end = sql.find("SET")?;
        let table_name = sql[table_start..table_end].trim().to_string();

        let set_start = table_end + "SET".len();
        let set_query = sql[set_start..].trim().to_string();

        Some(UpdateDefinition {
            table_name,
            set_query,
        })
    }

    pub fn load_data(&self) -> io::Result<HashMap<String, Vec<Value>>> {

        let table_def = read_table_definition(self.table_name.as_str()).unwrap();

        let all_rows: Vec<HashMap<String, Value>> = read_vec_of_bytes_from_file(self.table_name.as_str()).unwrap();

        let mut column_map: HashMap<String, Vec<Value>> = HashMap::new();

        for row in all_rows {
            for (key, value) in row {
                let key_is_number = table_def
                    .columns
                    .iter()
                    .any(|i| (i.data_type == DataType::Float || i.data_type == DataType::Integer) && i.name == key);
                if key_is_number {
                    column_map
                        .entry(key)
                        .or_insert_with(Vec::new)
                        .push(value);
                }
            }
        }

        let mut binding_counter = 0;

        let wgsl_declarations: Vec<String> = column_map.keys()
            .map(|key| {
                let column_type = table_def
                    .columns
                    .iter()
                    .find(|col| col.name == *key)
                    .map(|col| match col.data_type {
                        DataType::Integer => "array<i32>",
                        DataType::Float => "array<f32>",
                        _ => "array<unknown>",
                    })
                    .unwrap();

                let declaration = format!(
                    "@group(0)\n@binding({})\nvar<storage, read_write> {}: {};",
                    binding_counter, key, column_type
                );

                binding_counter += 1;
                declaration
            })
            .collect();

        let wgsl_code_header = wgsl_declarations.join("\n\n");

        let assignments = match ExpressionParser::parse(&*self.set_query) {
            Ok(intermediary_code) => {
                intermediary_code.assignments
            }
            Err(err) => panic!("Failed to parse intermediary code {}", err),
        };

        let statements = assignments
            .iter()
            .map(|assignment| {
                let mut adjusted_expression = assignment.expression.clone();
                for key in column_map.keys() {
                    let pattern = format!(r"\b{}\b", key);
                    let replacement = format!("{}[sys_index]", key);

                    let regex = Regex::new(&pattern).unwrap();
                    adjusted_expression = regex.replace_all(&adjusted_expression, replacement.as_str()).to_string();
                }

                if column_map.contains_key(&assignment.variable) {
                    format!(
                        "{}[sys_index] = {};",
                        assignment.variable,
                        adjusted_expression
                    )
                } else {
                    format!(
                        "let {} = {};",
                        assignment.variable,
                        adjusted_expression
                    )
                }
            })
            .collect::<Vec<String>>()
            .join("\n");

        let total_wgsl_code = wgsl_code_header + "
@compute
@workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>,
        @builtin(local_invocation_id) lid: vec3<u32>,
        @builtin(workgroup_id) wid: vec3<u32>) {
    let sys_index = wid.x * 64u + lid.x;
    if (sys_index < arrayLength(&" + column_map.keys().next().unwrap() + ")) {"
        + &statements +
"    }
}";

        let new_vals = ShaderExecutor.main(total_wgsl_code, column_map, table_def);

        let reconstructed_rows = reconstruct_rows(new_vals.clone());

        let updated_data: Vec<Vec<Vec<u8>>> = reconstructed_rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|(_, value)| serialize_from_value(value).unwrap())
                    .collect()
            })
            .collect();

        write_vec_of_bytes_to_file(updated_data, self.table_name.as_str(), false);

        let reconstructed_rows = reconstruct_rows(new_vals.clone());
        print_table(reconstructed_rows);

        let empty_map: HashMap<String, Vec<Value>> = HashMap::new();

        Ok(empty_map)
    }

    pub fn execute() {
        // TODO: Separate into execute and load
    }
}