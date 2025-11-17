use chardetng::EncodingDetector;

pub fn detect_and_decode(bytes: &[u8]) -> (String, &'static str) {
    // Detect encoding
    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);

    // Decode bytes
    let (decoded, _, _) = encoding.decode(bytes);

    //println!("Detected encoding: {}, had_errors: {}", encoding.name(), had_errors);
    (decoded.into_owned(), encoding.name())
}
