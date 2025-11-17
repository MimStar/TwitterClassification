use crate::csv_ext::cols_sniffer::ColsSniffer;

impl ColsSniffer {
    pub(super) fn bytes_is_rating(bytes: &[u8]) -> bool {
        let mut bytes_iter = bytes.iter();
        
        // Can't be empty
        let first_byte = match bytes_iter.next() {
            Some(b) => b,
            None => return false,
        };
    
        // It can be a single byte not surrounded by quotes
        let second_byte = match bytes_iter.next() {
            Some(b) => b,
            None => return Self::byte_is_rating(*first_byte),
        };
        
    
        // If there is more than one byte, there should be exactly 3 - [",x,"]
        let third_byte = match bytes_iter.next() {
            Some(b) => b,
            None => return false,
        };
        
        if let Some(_) = bytes_iter.next() {return false;}
    
        // check the three bytes are of format [",x,"]
        return *first_byte == b'"' && Self::byte_is_rating(*second_byte) && *third_byte == b'"';
    }
    
    pub(super) fn byte_is_rating(byte: u8) -> bool {
        match byte {
            b'0' | b'2' | b'4' => true,
            _ => false,
        }
    }
}