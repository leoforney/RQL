use std::fs::File;
use std::io;
use std::io::Read;
use crate::types::types::TableDefinition;

pub fn read_table_definition(table_name: &str) -> io::Result<TableDefinition> {
    let table_name_lowercase = table_name.to_lowercase() + "_def.bin";
    let file_path = format!("schema/{}", table_name_lowercase);

    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let table_definition: TableDefinition = bincode::deserialize(&buffer).unwrap();
    Ok(table_definition)
}

pub fn read_vec_of_bytes_from_file(table_name: &str) -> io::Result<Vec<Vec<u8>>> {
    let table_name_lowercase = table_name.to_lowercase() + "_data.bin";
    let file_path = format!("data/{}", table_name_lowercase);
    let mut file = File::open(file_path)?;
    let mut data = Vec::new();

    loop {
        let mut size_buf = [0u8; 8];
        if file.read_exact(&mut size_buf).is_err() {
            break;
        }
        let size = u64::from_le_bytes(size_buf) as usize;

        let mut bytes = vec![0u8; size];
        file.read_exact(&mut bytes)?;

        data.push(bytes);
    }

    Ok(data)
}
