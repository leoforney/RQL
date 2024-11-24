use crate::types::types::{ColumnDefinition, DataType, TableDefinition};

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
        let columns_end = sql.len() - 2; // Exclude the final ');'
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