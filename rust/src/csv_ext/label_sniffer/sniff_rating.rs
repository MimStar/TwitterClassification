use crate::csv_ext::label_sniffer::{AutoColumns, LabelSniffer, error::AutoLabelError};

impl LabelSniffer {
    pub(super) fn sniff_rating_with_data(
        veced_records: &mut [Vec<Vec<u8>>],
        data_column: usize
    ) -> Result<AutoColumns, AutoLabelError> {
        match Self::sniff_rating(&veced_records) {
            Some(rating_column) =>
                Ok(AutoColumns {data_column, rating_column}),
            
            None =>
                Err(AutoLabelError::NoRatingFound { data_column }),
        }
    }

    // Tries to infer which column contains the rating.
    // That is, a column where all content is always either 0, 2 or 4.
    pub(super) fn sniff_rating(records: &[Vec<Vec<u8>>]) -> Option<usize> {
        records
            .iter()
            .enumerate()
            .find(|(_, column)| {
                column
                    .iter()
                    .all(|field| {
                        Self::bytes_is_rating(field)
                    })
            })
            .and_then(|(i, _)| Some(i))
    }
}