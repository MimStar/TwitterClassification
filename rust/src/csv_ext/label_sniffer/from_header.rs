use csv::StringRecord;

use crate::csv_ext::label_sniffer::{AutoColumns, LabelSniffer};
use crate::csv_ext::label_sniffer::error::AutoLabelError;
use crate::csv_ext::label_sniffer::config;

impl LabelSniffer {
    pub fn sniff_labels_from_headers(
        headers: &StringRecord
    ) -> Result<AutoColumns, AutoLabelError> {
        let mut cols = AutoColumns {data_column: 0, rating_column: 0};
        let mut data_found = false;
        let mut rating_found = false;
    
        headers
            .iter()
            .enumerate()
            .for_each(|(i, header)| {
                let lower_header = header.to_lowercase();
                
                if config::DATA_TARGET_HEADERS.contains(&lower_header.as_str()) {
                    cols.data_column = i;
                    if rating_found {
                        return;
                    }
                    data_found = true;
                } else if config::RATING_TARGET_HEADERS.contains(&lower_header.as_str()) {
                    cols.rating_column = i;
                    if data_found {
                        return;
                    }
                    rating_found = true;
                }
            });
        
        if data_found && rating_found {
            return Ok(cols);
        }
    
        if data_found {
            return Err(AutoLabelError::NoRatingFound { data_column: cols.data_column })
        }
    
        if rating_found {
            return Err(AutoLabelError::NoDataFound { rating_column: cols.rating_column })
        }
    
        return Err(AutoLabelError::NoLabelFound);
    }
}