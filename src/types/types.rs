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

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct TableDefinition {
    pub(crate) name: String,
    pub(crate) columns: Vec<ColumnDefinition>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InsertDefinition {
    pub(crate) name: String,
    #[serde(skip)]
    pub(crate) table_definition: TableDefinition,
    pub(crate) values: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropDefinition {
    pub(crate) name: String,
}