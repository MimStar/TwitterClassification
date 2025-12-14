use godot::global::godot_print;

use crate::csv_ext::cols_sniffer::{AutoColumns, AutoColumnsError, ColsSniffer};

impl ColsSniffer {
    pub(super) fn sniff_columns_with_err(
        veced_records: &mut [Vec<Vec<u8>>],
        error: AutoColumnsError
    ) -> Result<AutoColumns, AutoColumnsError> {
        match error {
            AutoColumnsError::NoRatingFound { data_column } =>
                Self::sniff_rating_with_data(veced_records, data_column),

            AutoColumnsError::NoDataFound { rating_column } =>
                Self::sniff_data_with_rating(veced_records, rating_column),

            _ => Self::sniff_columns_from_vec2d(veced_records),
        }
    }

    pub(super) fn sniff_columns_from_vec2d(veced_records: &mut [Vec<Vec<u8>>]) -> Result<AutoColumns, AutoColumnsError> {
        let data_column = Self::sniff_data(veced_records);
        let rating_column = Self::sniff_rating(veced_records);
    
        if let Some(data_column) = data_column
        && let Some(rating_column) = rating_column {
            return Ok(AutoColumns { data_column, rating_column });
        }
    
        if let Some(data_column) = data_column {
            return Err(AutoColumnsError::NoRatingFound { data_column });
        }
    
        if let Some(rating_column) = rating_column {
            return Err(AutoColumnsError::NoDataFound { rating_column });
        }
    
        return Err(AutoColumnsError::NoColumnFound);
    }
}