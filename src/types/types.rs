use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum DataType {
    Integer,
    Float,
    Text,
    Boolean,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnDefinition {
    pub(crate) name: String,
    pub(crate) data_type: DataType,
    pub(crate) nullable: bool,
    pub(crate) unique: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TableDefinition {
    pub(crate) name: String,
    pub(crate) columns: Vec<ColumnDefinition>,
}