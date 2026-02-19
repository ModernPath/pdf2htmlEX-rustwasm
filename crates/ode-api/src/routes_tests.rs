#[cfg(test)]
mod integration_tests {
    use crate::routes::is_valid_pdf;

    #[test]
    fn test_is_valid_pdf() {
        assert!(is_valid_pdf(b"%PDF-1.4 test"));
        assert!(!is_valid_pdf(b"Not a PDF"));
        assert!(!is_valid_pdf(b""));
        assert!(!is_valid_pdf(b"%PDF")); // too short
    }

    #[test]
    fn test_is_valid_pdf_edge_cases() {
        assert!(is_valid_pdf(b"%PDF-2.0"));
        assert!(!is_valid_pdf(b"\x00\x00\x00\x00\x00"));
    }
}
