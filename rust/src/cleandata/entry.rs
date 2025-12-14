use crate::cleandata::CleanData;
use crate::cleandata::error::CleanDataError;

use crate::csv_ext::cols_sniffer::{self, ColsSniffer};

mod rules_regex;
mod auto_rules;

const DATA_COL: usize = 1;

impl CleanData {
    pub(super) fn clean_data_body(&mut self, data_path: &str) -> Result<String, CleanDataError> {
        // AUTO FILTERS GENERATION
        let filters = auto_rules::filters()?;

        // Columns sniffing
        // Warning here, rating and data cols might end up being the same
        let auto_columns = ColsSniffer::sniff_columns(data_path);
        let auto_columns = cols_sniffer::error::to_auto_columns(&auto_columns, DATA_COL);
        
        // CALL WITH GENERATED FILTERS AND STATIC COLUMNS
        self.clean_data_generic(
            data_path,
            "clean_data_temp.csv", 
            auto_columns.data_column, 
            auto_columns.rating_column, 
            &filters
        )
    }
}