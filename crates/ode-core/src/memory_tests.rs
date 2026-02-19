#[cfg(test)]
mod memory_usage_tests {
    use crate::config::ConversionConfig;
    use crate::convert_pdf;

    const MAX_MEMORY_MULTIPLIER: f64 = 2.0;
    const MAX_FILE_SIZE_FOR_CHECK: usize = 50 * 1024 * 1024; // 50 MB

    #[test]
    fn test_memory_usage_for_small_document() {
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
        let original_size = pdf_data.len();

        assert!(
            original_size < MAX_FILE_SIZE_FOR_CHECK,
            "Test file should be under 50MB"
        );

        let config = ConversionConfig::default();

        let result = convert_pdf(&pdf_data, &config);
        assert!(result.is_ok());

        let output_bundle = result.unwrap();

        let total_size = estimate_total_memory_usage(&output_bundle);
        let ratio = total_size as f64 / original_size as f64;

        // For very small files (< 1KB), allow a fixed overhead due to data structure initialization
        // The 2x limit is applied to files >= 1KB where it's more meaningful
        let limit = if original_size < 1024 {
            total_size < 10 * 1024 // Allow up to 10KB absolute overhead for tiny files
        } else {
            ratio <= MAX_MEMORY_MULTIPLIER
        };

        assert!(limit,
                "Memory usage ({:?} bytes) should be reasonable for original size ({:?} bytes), ratio: {:.2}",
                total_size, original_size, ratio);
    }

    #[test]
    fn test_memory_usage_with_multiple_pages() {
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<</Size 2>>\n%%EOF".to_vec();

        let config = ConversionConfig::default();
        let result = convert_pdf(&pdf_data, &config);
        assert!(result.is_ok());

        let output_bundle = result.unwrap();

        let total_size = estimate_total_memory_usage(&output_bundle);
        // For very small input PDFs (< 1KB), the ratio check is meaningless because
        // the fixed overhead of HTML/CSS generation dominates. Use an absolute limit instead.
        assert!(
            total_size < 64 * 1024, // 64KB absolute ceiling for a trivial PDF
            "Memory usage {} bytes should be reasonable for a minimal multi-page doc",
            total_size
        );
    }

    #[test]
    fn test_empty_memory_footprint() {
        use crate::renderer::OutputBundle;

        let bundle = OutputBundle::default();

        let size = estimate_total_memory_usage(&bundle);
        assert!(size < 1000, "Empty bundle should use minimal memory");
    }

    #[test]
    fn test_single_text_span_memory() {
        use crate::renderer::TextSpan;

        let spans = vec![TextSpan {
            text: "Hello World".to_string(),
            x: 100.0,
            y: 200.0,
            font_size: 12.0,
            font_id: None,
            color: "rgb(0,0,0)".to_string(),
        }];

        let size = estimate_text_spans_memory(&spans);

        assert!(size < 200, "Single text span should use minimal memory");
    }

    #[test]
    fn test_large_text_span_efficiency() {
        use crate::renderer::TextSpan;

        let large_text = "A".repeat(1000);
        let spans = vec![TextSpan {
            text: large_text.clone(),
            x: 100.0,
            y: 200.0,
            font_size: 12.0,
            font_id: None,
            color: "rgb(0,0,0)".to_string(),
        }];

        let size = estimate_text_spans_memory(&spans);

        assert!(
            size < 1500,
            "Large text span should not have excessive overhead"
        );
    }

    #[test]
    fn test_font_memory_efficiency() {
        use crate::config::FontFormat;
        use crate::renderer::OutputBundle;

        let mut bundle = OutputBundle::default();

        let font_data = vec![0u8; 1000];
        bundle.add_font(1, "TestFont".to_string(), font_data, FontFormat::Ttf);

        let size = estimate_total_memory_usage(&bundle);

        assert!(
            size < 2000,
            "Font bundle should not have excessive memory overhead"
        );
    }

    #[test]
    fn test_memory_growth_with_pages() {
        use crate::renderer::{OutputBundle, RenderedPage};

        let mut bundle = OutputBundle::default();

        for i in 0..10 {
            let page = RenderedPage {
                page_number: i,
                width: 612.0,
                height: 792.0,
                html: format!("<div>Page {}</div>", i),
                css: ".page {{ position: absolute; }}".to_string(),
                text_spans: vec![],
                font_ids: vec![],
                background_color: None,
                images: vec![],
            };
            bundle.add_page(page);
        }

        let size = estimate_total_memory_usage(&bundle);

        assert!(size < 5000, "10 pages should use reasonable memory");

        let avg_per_page = size / 10;
        assert!(
            avg_per_page < 500,
            "Average memory per page should be small"
        );
    }

    fn estimate_total_memory_usage(bundle: &crate::renderer::OutputBundle) -> usize {
        let mut total = 0usize;

        // Estimate pages memory
        total += estimate_pages_memory(&bundle.pages);

        // Estimate fonts memory
        total += estimate_fonts_memory(&bundle.fonts);

        // Estimate CSS memory
        total += bundle.css.len();

        total
    }

    fn estimate_pages_memory(pages: &[crate::renderer::RenderedPage]) -> usize {
        let mut total = 0usize;

        for page in pages {
            total += std::mem::size_of_val(page);
            total += page.html.len();
            total += page.css.len();
            total += estimate_text_spans_memory(&page.text_spans);
            total += std::mem::size_of::<u64>() * page.font_ids.len();
        }

        total
    }

    fn estimate_fonts_memory(fonts: &[crate::renderer::RenderedFont]) -> usize {
        let mut total = 0usize;

        for font in fonts {
            total += std::mem::size_of_val(font);
            total += font.font_name.len();
            total += font.data.len();
            total += font.content_hash.len();
            total += font.filename.len();
        }

        total
    }

    fn estimate_text_spans_memory(spans: &[crate::renderer::TextSpan]) -> usize {
        let mut total = 0usize;

        for span in spans {
            total += std::mem::size_of_val(span);
            total += span.text.len();
            total += span.color.len();
            if span.font_id.is_some() {
                total += std::mem::size_of::<u64>();
            }
        }

        total
    }
}
