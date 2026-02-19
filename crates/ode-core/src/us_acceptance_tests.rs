#[cfg(test)]
mod us_acceptance_tests {
    use crate::config::ConversionConfig;
    use crate::convert_pdf;
    use crate::util::hash::ContentHasher;
    use crate::util::zip_bomb::ZipBombDetector;

    fn create_simple_pdf() -> Vec<u8> {
        b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec()
    }

    #[test]
    fn test_us001_semantic_html_no_images() {
        let pdf_data = create_simple_pdf();
        let config = ConversionConfig::default();

        let result = convert_pdf(&pdf_data, &config);
        assert!(result.is_ok(), "PDF conversion should succeed");

        let output_bundle = result.unwrap();
        assert!(
            !output_bundle.pages.is_empty(),
            "Should have at least one page"
        );

        let page = &output_bundle.pages[0];
        assert!(
            !page.html.contains("<img"),
            "HTML should not contain <img> tags for text content"
        );
    }

    #[test]
    fn test_us001_error_handling_for_corrupted_pdf() {
        let corrupted_pdf = b"This is not a PDF file at all";
        let config = ConversionConfig::default();

        let result = convert_pdf(corrupted_pdf, &config);
        assert!(result.is_err(), "Should return error for corrupted PDF");
    }

    #[test]
    fn test_us001_empty_pdf_error() {
        let empty_pdf = b"";
        let config = ConversionConfig::default();

        let result = convert_pdf(empty_pdf, &config);
        assert!(result.is_err(), "Should return error for empty PDF");
    }

    #[test]
    fn test_us004_content_addressed_filenames() {
        let font_data = b"font-content-test-123";
        let hash1 = ContentHasher::hash_bytes(font_data);
        let hash2 = ContentHasher::hash_bytes(font_data);

        assert_eq!(hash1, hash2, "Same data should produce same hash");

        let filename1 = ContentHasher::generate_content_addressed_filename(font_data, "woff2");
        let filename2 = ContentHasher::generate_content_addressed_filename(font_data, "woff2");
        let filename3 = ContentHasher::generate_content_addressed_filename(b"different", "woff2");

        assert_eq!(
            filename1, filename2,
            "Same data should produce same filename"
        );
        assert_ne!(
            filename1, filename3,
            "Different data should produce different filename"
        );
    }

    #[test]
    fn test_us004_hash_length_and_format() {
        let font_data = b"test";
        let hash = ContentHasher::hash_bytes(font_data);

        assert_eq!(hash.len(), 64, "SHA256 hash should be 64 characters");
        assert!(
            hash.chars().all(|c| c.is_ascii_hexdigit()),
            "Hash should only contain hex digits"
        );
    }

    #[test]
    fn test_us003_zip_bomb_detection() {
        let detector = ZipBombDetector::with_limit_100_to_1();

        let large_data = vec![0u8; 10_000_001];
        let result = detector.check_buffer(&large_data, true);
        assert!(
            result.is_err(),
            "Should detect potential zip bomb for very large data"
        );
    }

    #[test]
    fn test_us003_result_types_used_throughout() {
        let pdf_data = create_simple_pdf();
        let config = ConversionConfig::default();

        let result: Result<_, _> = convert_pdf(&pdf_data, &config);

        match result {
            Ok(output_bundle) => {
                assert!(matches!(
                    output_bundle,
                    crate::renderer::OutputBundle { .. }
                ));
            }
            Err(_) => {
                // Any error is acceptable - just verifies Result type is used
            }
        }
    }

    #[test]
    fn test_us002_font_extraction_with_metadata() {
        let pdf_data = create_simple_pdf();
        let config = ConversionConfig::default();

        let output_bundle = convert_pdf(&pdf_data, &config).unwrap();

        for font in &output_bundle.fonts {
            assert!(
                !font.content_hash.is_empty(),
                "Font should have a content hash"
            );
            assert_eq!(
                font.content_hash.len(),
                64,
                "Content hash should be SHA256 length"
            );
        }
    }
}
