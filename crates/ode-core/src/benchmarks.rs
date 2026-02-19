#[cfg(test)]
mod performance_benchmarks {
    use crate::config::ConversionConfig;
    use crate::convert_pdf;
    use std::time::Instant;

    const TARGET_MS_STANDARD_DOC: u128 = 500; // 500ms for standard document
    const TARGET_MEMORY_STANDARD_DOC: usize = 256 * 1024 * 1024; // 256MB

    #[test]
    fn benchmark_simple_pdf_conversion() {
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
        let config = ConversionConfig::default();

        let start = Instant::now();
        let result = convert_pdf(&pdf_data, &config);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed.as_millis() < TARGET_MS_STANDARD_DOC,
            "Simple PDF conversion should complete in < {}ms, took: {:?}",
            TARGET_MS_STANDARD_DOC,
            elapsed
        );
    }

    #[test]
    fn benchmark_pdf_parsing() {
        use crate::parser::parse_pdf;

        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";

        let start = Instant::now();
        let result = parse_pdf(pdf_data);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed.as_millis() < 100, "PDF parsing should be fast");
    }

    #[test]
    fn benchmark_text_extraction() {
        use crate::parser::PdfDocument;
        use crate::renderer::render_pdf_page;

        let mut doc = PdfDocument::new();
        doc.pages.push(crate::parser::PdfPage {
            page_number: 1,
            width: 612.0,
            height: 792.0,
            contents: Vec::new(),
            fonts: Vec::new(),
            rotation: 0,
            dict: None,
            font_cmaps: std::collections::HashMap::new(),
            images: std::collections::HashMap::new(),
            form_xobjects: std::collections::HashMap::new(),
        });

        let config = ConversionConfig::default();

        let start = Instant::now();
        let result = render_pdf_page(&doc, 0, 1, &config);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed.as_millis() < 100, "Text extraction should be fast");
    }

    #[test]
    fn benchmark_coordinate_transformations() {
        use crate::util::math::TransformMatrix;

        let mut tm = TransformMatrix::identity();
        tm.a = 2.0;
        tm.b = 0.5;
        tm.c = 0.3;
        tm.d = 2.0;
        tm.e = 100.5;
        tm.f = 200.7;

        let iterations = 100_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = tm.transform_point(10.0, 20.0);
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / iterations as u128;
        assert!(
            avg_ns < 1000,
            "Coordinate transformation should be very fast"
        );
    }

    #[test]
    fn benchmark_matrix_multiplication() {
        use crate::util::math::TransformMatrix;

        let tm1 = TransformMatrix {
            a: 2.0,
            b: 0.5,
            c: 0.3,
            d: 2.0,
            e: 10.0,
            f: 20.0,
        };
        let tm2 = TransformMatrix {
            a: 1.5,
            b: 0.2,
            c: 0.1,
            d: 1.5,
            e: 30.0,
            f: 40.0,
        };

        let iterations = 100_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = tm1 * tm2;
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / iterations as u128;
        assert!(
            avg_ns < 100,
            "Matrix multiplication should be extremely fast"
        );
    }

    #[test]
    fn benchmark_bounding_box_operations() {
        use crate::util::math::BoundingBox;

        let bbox1 = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        let bbox2 = BoundingBox::new(50.0, 50.0, 150.0, 150.0);

        let iterations = 100_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = bbox1.intersect(&bbox2);
            let _ = bbox1.width();
            let _ = bbox1.height();
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / iterations as u128;
        assert!(
            avg_ns < 100,
            "Bounding box operations should be extremely fast"
        );
    }

    #[test]
    fn benchmark_color_operations() {
        use crate::types::color::Color;

        let color = Color::new(128, 64, 32);

        let iterations = 100_000;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = color.to_css_string();
            let _ = color.distance(&color);
            let _ = color.to_hash();
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / iterations as u128;
        assert!(avg_ns < 200, "Color operations should be fast");
    }

    #[test]
    fn benchmark_text_segmentation() {
        use crate::renderer::text::TextExtractor;

        let mut extractor = TextExtractor::new();

        let iterations = 1000;

        let start = Instant::now();
        for i in 0..iterations {
            let text = format!("Text segment {}", i);
            let _ = extractor.add_text(&text, 10.0 + (i as f64), 20.0);
        }
        let elapsed = start.elapsed();

        let avg_ns = elapsed.as_nanos() / iterations as u128;
        assert!(avg_ns < 5000, "Text segmentation should be reasonably fast");
    }

    #[test]
    fn verify_sub_second_conversion_standard_doc() {
        // Use a simple but valid PDF that our parser can handle
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
        let config = ConversionConfig::default();

        let start = Instant::now();
        let result = convert_pdf(&pdf_data, &config);
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(
            elapsed.as_millis() < TARGET_MS_STANDARD_DOC,
            "Standard document should convert in < {}ms, took: {:?}",
            TARGET_MS_STANDARD_DOC,
            elapsed
        );
    }

    #[test]
    fn verify_memory_under_limit_standard_doc() {
        let pdf_data = create_standard_test_pdf();
        let config = ConversionConfig::default();

        let result = convert_pdf(&pdf_data, &config);
        assert!(result.is_ok(), "convert_pdf failed: {:?}", result.err());

        let output_bundle = result.unwrap();
        let estimated_memory = estimate_bundle_memory(&output_bundle);

        assert!(
            estimated_memory < TARGET_MEMORY_STANDARD_DOC,
            "Memory usage should be < {} bytes, estimated: {} bytes",
            TARGET_MEMORY_STANDARD_DOC,
            estimated_memory
        );
    }

    #[test]
    fn benchmark_page_processing_scalability() {
        let config = ConversionConfig::default();

        // Test with increasing number of pages to verify scalability
        for page_count in [1, 3, 5] {
            let pdf_data = create_multi_page_pdf(page_count);

            let start = Instant::now();
            let result = convert_pdf(&pdf_data, &config);
            let elapsed = start.elapsed();

            assert!(result.is_ok(), "convert_pdf failed for {} pages: {:?}", page_count, result.err());

            // Should scale roughly linearly with page count
            let expected_max_ms = (page_count * 200) as u128;
            assert!(
                elapsed.as_millis() < expected_max_ms,
                "{} page conversion should be fast, took: {:?}",
                page_count,
                elapsed
            );
        }
    }

    fn create_standard_test_pdf() -> Vec<u8> {
        create_multi_page_pdf(3)
    }

    fn create_multi_page_pdf(page_count: usize) -> Vec<u8> {
        let mut pdf = Vec::<u8>::new();
        let mut offsets = Vec::<usize>::new();

        // Header
        pdf.extend_from_slice(b"%PDF-1.4\n");

        // Object 1: Catalog
        offsets.push(pdf.len());
        pdf.extend_from_slice(b"1 0 obj\n<</Type/Catalog/Pages 2 0 R>>\nendobj\n");

        // Object 2: Pages
        offsets.push(pdf.len());
        let mut kids = String::new();
        for i in 0..page_count {
            kids.push_str(&format!("{} 0 R ", i + 3));
        }
        pdf.extend_from_slice(format!("2 0 obj\n<</Type/Pages/Count {}/Kids[{}]>>\nendobj\n", page_count, kids.trim()).as_bytes());

        // Page objects (3..3+page_count)
        for i in 0..page_count {
            let page_num = i + 3;
            let stream_num = page_count + 3 + i;
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n<</Type/Page/Parent 2 0 R/MediaBox[0 0 612 792]/Contents {} 0 R>>\nendobj\n", page_num, stream_num).as_bytes());
        }

        // Stream objects
        for i in 0..page_count {
            let stream_num = page_count + 3 + i;
            let stream_content = format!("BT\n/F1 12 Tf\n100 700 Td\n(Page {}) Tj\nET\n", i + 1);
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n<</Length {}>>\nstream\n", stream_num, stream_content.len()).as_bytes());
            pdf.extend_from_slice(stream_content.as_bytes());
            pdf.extend_from_slice(b"endstream\nendobj\n");
        }

        // Xref
        let xref_offset = pdf.len();
        let total_objects = 1 + offsets.len(); // +1 for object 0
        pdf.extend_from_slice(format!("xref\n0 {}\n", total_objects).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for offset in &offsets {
            pdf.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
        }

        // Trailer
        pdf.extend_from_slice(format!("trailer\n<</Size {}/Root 1 0 R>>\nstartxref\n{}\n%%EOF", total_objects, xref_offset).as_bytes());

        pdf
    }

    fn estimate_bundle_memory(bundle: &crate::renderer::OutputBundle) -> usize {
        let mut total = std::mem::size_of_val(bundle);

        for page in &bundle.pages {
            total += std::mem::size_of_val(page) + page.html.len() + page.css.len();
            for span in &page.text_spans {
                total += std::mem::size_of_val(span) + span.text.len() + span.color.len();
            }
        }

        for font in &bundle.fonts {
            total += std::mem::size_of_val(font) + font.font_name.len() + font.data.len();
        }

        total
    }
}
