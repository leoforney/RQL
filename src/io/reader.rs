use std::fs::File;
use std::io::Read;
use crate::types::types::TableDefinition;

pub fn read_table() -> std::io::Result<TableDefinition> {
    let mut file = File::open("data.bin")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let decoded: TableDefinition = bincode::deserialize(&buffer).unwrap();
    Ok(decoded)
}