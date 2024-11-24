mod types;
mod query;
mod io;

use crate::io::writer::write_table;
use crate::types::types::TableDefinition;
use serde::{Deserialize, Serialize};
use std::io::{Read, Result, Write};
use crate::io::reader::read_table;


fn main() -> Result<()> {
    let sql = r#"
        CREATE TABLE users (
            id INTEGER NOT NULL UNIQUE,
            name TEXT NOT NULL,
            is_active BOOLEAN
        );
    "#;

    if let Some(table) = TableDefinition::from_sql(sql) {
        println!("Parsed Table Definition: {:?}", table);

        write_table(&table).expect("Failed to write");

        println!("Reconstructed SQL:\n{}", table.to_sql());
    } else {
        println!("Failed to parse SQL");
    }

    let read_table = read_table().unwrap();
    println!("Parsed Table: {:}", read_table.to_sql());

    Ok(())
}
