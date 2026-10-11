use super::MAX_TABLE_COLUMN_WIDTH;
use crate::connectors;
use crate::logger;
use crate::table::Table;

pub fn execute() {
    let rows = connectors::CONNECTORS
        .iter()
        .map(|connector| {
            vec![
                connector.key.to_string(),
                (connector.connection_string_hint)(),
            ]
        })
        .collect();
    logger::info(
        Table::new()
            .header(vec![
                "Type".to_string(),
                "Connection string hint".to_string(),
            ])
            .body(rows)
            .max_col_width(MAX_TABLE_COLUMN_WIDTH)
            .padding(1)
            .indent(2)
            .border(true)
            .to_string(),
    );
}
