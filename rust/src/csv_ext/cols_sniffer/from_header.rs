use csv::StringRecord;

use crate::csv_ext::cols_sniffer::{AutoColumns, ColsSniffer};
use crate::csv_ext::cols_sniffer::error::AutoColumnsError;
use crate::csv_ext::cols_sniffer::config;

impl ColsSniffer {
    pub(super) fn sniff_columns_from_headers(
        headers: &StringRecord
    ) -> Result<AutoColumns, AutoColumnsError> {
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
            return Err(AutoColumnsError::NoRatingFound { data_column: cols.data_column })
        }
    
        if rating_found {
            return Err(AutoColumnsError::NoDataFound { rating_column: cols.rating_column })
        }
    
        return Err(AutoColumnsError::NoColumnFound);
    }
}