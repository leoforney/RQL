#[cfg(test)]
pub mod tests {
    use regex::Regex;
    use crate::io::writer::write_table_definition;
    use crate::types::types::TableDefinition;

    fn reduce_spaces(input: &str) -> String {
        let re = Regex::new(r"\s{2,}").unwrap();
        re.replace_all(input, " ").to_string()
    }

    #[test]
    fn test_parse_create() {
        let sql = r#"
        CREATE TABLE users (
            id INTEGER NOT NULL UNIQUE,
            name TEXT NOT NULL,
            is_active BOOLEAN
            );
        "#;

        if let Some(table) = TableDefinition::from_sql(sql) {
            println!("Parsed Table Definition: {:?}", table);

            write_table_definition(&table).expect("Failed to write");

            assert_eq!(reduce_spaces(&*table.to_sql().replace("\n", "")).trim(), sql.replace("\n", "").replace("  ", " ").trim());
        } else {
            println!("Failed to parse SQL");
        }
    }
}