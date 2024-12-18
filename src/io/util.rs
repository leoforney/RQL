use crate::types::types::Value;
use prettytable::{format, Cell, Row, Table};
use std::collections::HashMap;

pub fn print_table(rows: Vec<HashMap<String, Value>>) {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    let mut column_order: Vec<String> = Vec::new();

    if let Some(first_row) = rows.get(0) {
        column_order = first_row.keys().cloned().collect();
        let headers: Vec<Cell> = column_order.iter().map(|key| Cell::new(key)).collect();
        table.set_titles(Row::new(headers));
    }

    for row in rows {
        let cells: Vec<Cell> = column_order
            .iter()
            .map(|key| {
                row.get(key)
                    .map(|value| Cell::new(&*value.to_string()))
                    .unwrap_or_else(|| Cell::new(""))
            })
            .collect();
        table.add_row(Row::new(cells));
    }

    table.printstd();
}

pub fn reconstruct_rows(
    column_map: HashMap<String, Vec<Value>>,
) -> Vec<HashMap<String, Value>> {
    if column_map.is_empty() {
        return vec![];
    }

    let num_rows = column_map.values().next().unwrap().len();
    for (key, column) in &column_map {
        assert_eq!(
            column.len(),
            num_rows,
            "Column '{}' has a different number of rows ({}) than expected ({})",
            key,
            column.len(),
            num_rows
        );
    }

    let mut all_rows: Vec<HashMap<String, Value>> = vec![HashMap::new(); num_rows];

    for (key, column) in column_map {
        for (row_idx, value) in column.into_iter().enumerate() {
            all_rows[row_idx].insert(key.clone(), value);
        }
    }

    all_rows
}
