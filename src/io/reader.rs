use crate::types::types::{DataType, TableDefinition, Value};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::{fmt, io};

pub fn read_table_definition(table_name: &str) -> io::Result<TableDefinition> {
    let table_name_lowercase = table_name.to_lowercase() + "_def.bin";
    let file_path = format!("schema/{}", table_name_lowercase);

    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let table_definition: TableDefinition = bincode::deserialize(&buffer).unwrap();
    Ok(table_definition)
}

pub fn read_vec_of_bytes_from_file(table_name: &str) -> io::Result<Vec<HashMap<String, Value>>> {
    let mut results = Vec::new();
    let table_name_lowercase = table_name.to_lowercase() + "_data.bin";
    let table_definition = read_table_definition(table_name)?;
    let file_path = format!("data/{}", table_name_lowercase);
    let mut file = File::open(file_path)?;

    loop {
        let mut start_marker = [0u8; 1];
        if file.read_exact(&mut start_marker).is_err() {
            break;
        }

        if start_marker[0] != 0xAB {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid start marker",
            ));
        }

        let mut size_buf = [0u8; 8];
        file.read_exact(&mut size_buf)?;
        let row_size = u64::from_le_bytes(size_buf);

        let mut end_marker = [0u8; 1];
        file.read_exact(&mut end_marker)?;
        if end_marker[0] != 0xCD {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid end marker",
            ));
        }

        let mut row_data = vec![0u8; row_size as usize];
        file.read_exact(&mut row_data)?;

        let mut cursor = &row_data[..];
        let mut inner_results_map = HashMap::new();
        for column in &table_definition.columns {
            let value = match column.data_type {
                DataType::Integer => {
                    let value: i32 = bincode::deserialize_from(&mut cursor).map_err(|e| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("Deserialization error: {}", e))
                    })?;
                    Value::Integer(value)
                }
                DataType::Float => {
                    let value: f64 = bincode::deserialize_from(&mut cursor).map_err(|e| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("Deserialization error: {}", e))
                    })?;
                    Value::Float(value)
                }
                DataType::Text => {
                    let value: String = bincode::deserialize_from(&mut cursor).map_err(|e| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("Deserialization error: {}", e))
                    })?;
                    Value::Text(value)
                }
                DataType::Boolean => {
                    let value: bool = bincode::deserialize_from(&mut cursor).map_err(|e| {
                        io::Error::new(io::ErrorKind::InvalidData, format!("Deserialization error: {}", e))
                    })?;
                    Value::Boolean(value)
                }
            };

            inner_results_map.insert(column.name.clone(), value);
        }

        results.push(inner_results_map);
    }

    Ok(results)
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Text(v) => write!(f, "{}", v),
            Value::Boolean(v) => write!(f, "{}", v),
        }
    }
}
