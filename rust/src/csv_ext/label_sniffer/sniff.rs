use crate::csv_ext::label_sniffer::{AutoColumns, AutoLabelError, LabelSniffer};

impl LabelSniffer {
    pub fn sniff_labels_with_err(
        veced_records: &mut [Vec<Vec<u8>>],
        error: AutoLabelError
    ) -> Result<AutoColumns, AutoLabelError> {
        match error {
            AutoLabelError::NoRatingFound { data_column } =>
                Self::sniff_rating_with_data(veced_records, data_column),

            AutoLabelError::NoDataFound { rating_column } =>
                Self::sniff_data_with_rating(veced_records, rating_column),

            _ => Self::sniff_labels_from_vec2d(veced_records),
        }
    }

    pub(super) fn sniff_labels_from_vec2d(veced_records: &mut [Vec<Vec<u8>>]) -> Result<AutoColumns, AutoLabelError> {
        let data_column = Self::sniff_data(veced_records);
        let rating_column = Self::sniff_rating(veced_records);
    
        if let Some(data_column) = data_column
        && let Some(rating_column) = rating_column {
            return Ok(AutoColumns { data_column, rating_column });
        }
    
        if let Some(data_column) = data_column {
            return Err(AutoLabelError::NoRatingFound { data_column });
        }
    
        if let Some(rating_column) = rating_column {
            return Err(AutoLabelError::NoDataFound { rating_column });
        }
    
        return Err(AutoLabelError::NoLabelFound);
    }
}