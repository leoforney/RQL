use std::fs::File;
use std::io::Write;
use crate::types::types::TableDefinition;

pub fn write_table(my_instance: &TableDefinition) -> std::io::Result<()> {
    let mut file = File::create("data.bin")?;
    let encoded: Vec<u8> = bincode::serialize(&my_instance).unwrap();
    file.write_all(&encoded)?;
    println!("Struct written to file.");
    Ok(())
}