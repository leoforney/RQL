use crate::types::types::TableDefinition;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{fs, io};

pub fn write_table_definition(table_definition: &TableDefinition) -> io::Result<()> {
    fs::create_dir_all("schema")?;

    let table_name_lowercase = table_definition.name.clone().to_lowercase() + "_def.bin";
    let file_path = format!("schema/{}", table_name_lowercase);

    let mut file = File::create(file_path)?;
    let encoded: Vec<u8> = bincode::serialize(&table_definition).unwrap();
    file.write_all(&encoded)?;

    Ok(())
}

pub fn append_vec_of_bytes_to_file(data: Vec<Vec<Vec<u8>>>, table_name: &str) -> io::Result<()> {
    fs::create_dir_all("data")?;

    let table_name_lowercase = table_name.to_lowercase() + "_data.bin";
    let file_path = format!("data/{}", table_name_lowercase);

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)?;

    for row in data {
        let mut row_data = Vec::new();

        for column in row {
            row_data.extend(column);
        }

        let row_size = row_data.len() as u64;
        
        file.write_all(&[0xAB])?;
        file.write_all(&row_size.to_le_bytes())?;
        file.write_all(&[0xCD])?;
        
        file.write_all(&row_data)?;
    }

    Ok(())
}