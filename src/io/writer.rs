use crate::types::types::{DataType, TableDefinition, Value};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{fs, io};
use std::collections::HashMap;

pub fn write_table_definition(table_definition: &TableDefinition) -> io::Result<()> {
    fs::create_dir_all("schema")?;

    let table_name_lowercase = table_definition.name.clone().to_lowercase() + "_def.bin";
    let file_path = format!("schema/{}", table_name_lowercase);

    let mut file = File::create(file_path)?;
    let encoded: Vec<u8> = bincode::serialize(&table_definition).unwrap();
    file.write_all(&encoded)?;

    Ok(())
}

pub fn update_rows(
    existing_rows: Vec<HashMap<String, Value>>,
    updates: Vec<HashMap<String, Value>>,
) -> Vec<HashMap<String, Value>> {
    existing_rows
        .into_iter()
        .enumerate()
        .map(|(index, mut row)| {
            if let Some(updated_row) = updates.get(index) {
                for (key, value) in updated_row {
                    row.insert(key.clone(), value.clone());
                }
            }
            row
        })
        .collect()
}

pub fn serialize_from_value(value: &Value) -> io::Result<Vec<u8>> {
    match value {
        Value::Integer(parsed) => Ok(bincode::serialize(parsed).unwrap()),
        Value::Float(parsed) => Ok(bincode::serialize(parsed).unwrap()),
        Value::Text(parsed) => Ok(bincode::serialize(parsed).unwrap()),
        Value::Boolean(parsed) => Ok(bincode::serialize(parsed).unwrap()),
    }
}

pub fn serialize_value(value: &str, data_type: &DataType) -> io::Result<Vec<u8>> {
    match data_type {
        DataType::Integer => {
            let parsed: i32 = value.parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid integer value")
            })?;
            Ok(bincode::serialize(&parsed).unwrap())
        }
        DataType::Float => {
            let parsed: f32 = value.parse::<f32>().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid float value")
            })?;
            Ok(bincode::serialize(&parsed).unwrap())
        }
        DataType::Text => Ok(bincode::serialize(&value).unwrap()),
        DataType::Boolean => {
            let parsed: bool = value.parse().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid boolean value")
            })?;
            Ok(bincode::serialize(&parsed).unwrap())
        }
    }
}



pub fn write_vec_of_bytes_to_file(data: Vec<Vec<Vec<u8>>>, table_name: &str, append: bool) -> io::Result<()> {
    fs::create_dir_all("data")?;

    let table_name_lowercase = table_name.to_lowercase() + "_data.bin";
    let file_path = format!("data/{}", table_name_lowercase);

    let mut file = OpenOptions::new()
        .append(append)
        .write(!append)
        .create(true)
        .truncate(!append)
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
