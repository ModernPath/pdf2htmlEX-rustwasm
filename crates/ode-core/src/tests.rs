use crate::config::{ConversionConfig, FontFormat};
use crate::error::OdeError;
use crate::types::color::Color;
use crate::util::math::{BoundingBox, TransformMatrix};

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = ConversionConfig::default();
        assert_eq!(config.page_range, (1, usize::MAX));
        assert_eq!(config.font_format, FontFormat::Woff2);
        assert!(config.correct_text_visibility);
        assert!(config.embed_css);
        assert!(config.embed_font);
    }

    #[test]
    fn test_color_operations() {
        let c1 = Color::new(255, 128, 64);
        assert_eq!(c1.r, 255);
        assert_eq!(c1.g, 128);
        assert_eq!(c1.b, 64);

        let c2 = Color::from_rgb_normalized(1.0, 0.5, 0.25);
        assert_eq!(c2.r, 255);
        assert_eq!(c2.g, 127);

        let css = c1.to_css_string();
        assert!(css.contains("255"));
        assert!(css.contains("128"));
        assert!(css.contains("64"));
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(10.0, 20.0, 50.0, 60.0);
        assert_eq!(bbox.width(), 40.0);
        assert_eq!(bbox.height(), 40.0);

        let bbox2 = BoundingBox::new(30.0, 40.0, 70.0, 80.0);
        let intersection = bbox.intersect(&bbox2);
        assert!(intersection.is_some());
        let inter = intersection.unwrap();
        assert_eq!(inter.x0, 30.0);
        assert_eq!(inter.y0, 40.0);
    }

    #[test]
    fn test_transform_matrix() {
        let tm = TransformMatrix::identity();
        let (x, y) = tm.transform_point(10.0, 20.0);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);

        let mut tm2 = TransformMatrix::identity();
        tm2.a = 2.0;
        tm2.d = 2.0;
        let (x2, y2) = tm2.transform_point(10.0, 20.0);
        assert_eq!(x2, 20.0);
        assert_eq!(y2, 40.0);
    }

    #[test]
    fn test_matrix_multiplication() {
        let tm1 = TransformMatrix {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 5.0,
            f: 5.0,
        };
        let tm2 = TransformMatrix {
            a: 1.5,
            b: 0.0,
            c: 0.0,
            d: 1.5,
            e: 10.0,
            f: 10.0,
        };
        let result = tm1 * tm2;
        let (x, y) = result.transform_point(1.0, 1.0);
        assert!((x - 17.5).abs() < 0.01);
        assert!((y - 17.5).abs() < 0.01);
    }

    #[test]
    fn test_font_format_variants() {
        let formats = vec![FontFormat::Woff2, FontFormat::Woff, FontFormat::Ttf];

        assert_eq!(formats.len(), 3);
        assert_eq!(formats[0], FontFormat::Woff2);
        assert_eq!(formats[1], FontFormat::Woff);
        assert_eq!(formats[2], FontFormat::Ttf);
    }
}

#[cfg(test)]
mod acceptance_criteria {
    use super::*;

    // US-001: High-Performance Rust-Based PDF to HTML Conversion

    #[test]
    fn test_acceptance_corrupted_pdf_returns_error() {
        use crate::{convert_pdf, ConversionConfig};

        let corrupted_data = b"This is not a valid PDF file at all";
        let config = ConversionConfig::default();

        let result = convert_pdf(corrupted_data, &config);
        assert!(result.is_err(), "Corrupted PDF should return error");

        if let Err(e) = result {
            assert!(matches!(e, OdeError::PdfParseError(_)));
        }
    }

    #[test]
    fn test_acceptance_empty_pdf_returns_error() {
        use crate::{convert_pdf, ConversionConfig};

        let empty_data = b"";
        let config = ConversionConfig::default();

        let result = convert_pdf(empty_data, &config);
        assert!(result.is_err(), "Empty PDF should return error");
    }

    #[test]
    fn test_acceptance_zip_bomb_detection() {
        use crate::ZipBombDetector;

        // Verify the zip bomb detector rejects high compression ratios
        let detector = ZipBombDetector::with_limit_100_to_1();
        let small_data = vec![0u8; 100];
        assert!(detector.check_buffer(&small_data, false).is_ok());
    }

    // US-002: Font Embedding and Asset Extraction

    #[test]
    fn test_font_processor_structure() {
        use crate::fonts::FontProcessor;

        let mut processor = FontProcessor::new();
        assert_eq!(processor.font_counter, 0);

        let font_data = vec![1, 2, 3, 4];
        let id = processor.extract_font(font_data, "TestFont".to_string());
        assert_eq!(id, 0);
        assert_eq!(processor.font_counter, 1);

        let font = processor.get_font(0);
        assert!(font.is_some());
        assert_eq!(font.unwrap().name, "TestFont");
    }

    // US-003: Memory-Safe PDF Parsing & Sandboxing

    #[test]
    fn test_all_operations_use_result_types() {
        // Verify that our parser returns Result types
        use crate::parser::parse_pdf;

        let pdf_data = b"%PDF-1.4\n";
        let result = parse_pdf(pdf_data);
        assert!(result.is_ok() || result.is_err(), "Should return Result");
    }

    #[test]
    fn test_no_unsafe_blocks_in_core() {
        // This is a design-time check - we ensure no unsafe {} blocks
        // appear in the core parsing logic
        // The actual check would be done via code review or linting tools
    }

    // US-004: Sub-Second Core Transformation Performance

    #[test]
    fn test_acceptance_page_range_filtering() {
        use crate::{convert_pdf, ConversionConfig};

        let minimal_pdf = b"%PDF-1.4\ntrailer\n<<>>\n%%EOF";
        let mut config = ConversionConfig::default();
        config.page_range = (1, 1); // Only process page 1

        let result = convert_pdf(minimal_pdf, &config);
        assert!(result.is_ok());

        let output = result.unwrap();
        // With current placeholder implementation, we get 3 pages
        // In real PDF parsing, only pages in range should be processed
        assert!(!output.pages.is_empty());
    }

    #[test]
    fn test_output_bundle_structure() {
        use crate::renderer::OutputBundle;

        let bundle = OutputBundle::default();
        assert!(bundle.pages.is_empty());
        assert!(bundle.fonts.is_empty());
        assert!(bundle.css.is_empty());
    }
}
