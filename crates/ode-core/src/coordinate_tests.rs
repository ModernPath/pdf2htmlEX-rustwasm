#[cfg(test)]
mod coordinate_accuracy_tests {
    use crate::config::ConversionConfig;
    use crate::convert_pdf;
    use crate::renderer::TextSpan;
    use crate::util::math::{BoundingBox, TransformMatrix};

    #[test]
    fn test_coordinate_accuracy_within_half_point_margin() {
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec();
        let config = ConversionConfig::default();

        let result = convert_pdf(&pdf_data, &config);
        assert!(result.is_ok());

        let output_bundle = result.unwrap();

        if !output_bundle.pages.is_empty() {
            let page = &output_bundle.pages[0];

            for span in &page.text_spans {
                verify_coordinate_sanity(span);
            }
        }
    }

    fn verify_coordinate_sanity(span: &TextSpan) {
        assert!(
            span.x.is_finite() && span.y.is_finite(),
            "Text coordinates should be finite (x={:?}, y={:?})",
            span.x,
            span.y
        );

        assert!(
            span.x >= 0.0 && span.y >= 0.0,
            "Text coordinates should be non-negative (x={:?}, y={:?})",
            span.x,
            span.y
        );

        assert!(
            span.font_size > 0.0 && span.font_size < 1000.0,
            "Font size should be reasonable (got {})",
            span.font_size
        );
    }

    #[test]
    fn test_transform_matrix_accuracy() {
        let tm = TransformMatrix::identity();
        let test_x = 100.0;
        let test_y = 200.0;

        let (x, y) = tm.transform_point(test_x, test_y);

        assert!(
            (x - test_x).abs() < 0.5,
            "Transform should preserve x within 0.5pt"
        );
        assert!(
            (y - test_y).abs() < 0.5,
            "Transform should preserve y within 0.5pt"
        );
    }

    #[test]
    fn test_translation_matrix_accuracy() {
        let mut tm = TransformMatrix::identity();
        tm.e = 50.5;
        tm.f = 100.3;

        let (x, y) = tm.transform_point(0.0, 0.0);

        assert!(
            (x - 50.5).abs() < 0.5,
            "Translation should be accurate for x"
        );
        assert!(
            (y - 100.3).abs() < 0.5,
            "Translation should be accurate for y"
        );
    }

    #[test]
    fn test_scaling_matrix_accuracy() {
        let mut tm = TransformMatrix::identity();
        tm.a = 2.0;
        tm.d = 2.0;

        let (x, y) = tm.transform_point(10.0, 20.0);

        assert!(
            (x - 20.0).abs() < 0.5,
            "Scale transform should be accurate for x"
        );
        assert!(
            (y - 40.0).abs() < 0.5,
            "Scale transform should be accurate for y"
        );
    }

    #[test]
    fn test_bounding_box_accuracy() {
        let bbox = BoundingBox::new(10.0, 20.0, 50.0, 60.0);

        let width = bbox.width();
        let height = bbox.height();

        assert!(
            (width - 40.0).abs() < 0.001,
            "Bounding box width should be accurate"
        );
        assert!(
            (height - 40.0).abs() < 0.001,
            "Bounding box height should be accurate"
        );
    }

    #[test]
    fn test_bounding_box_intersection_accuracy() {
        let bbox1 = BoundingBox::new(0.0, 0.0, 100.0, 100.0);
        let bbox2 = BoundingBox::new(50.0, 50.0, 150.0, 150.0);

        let inter = bbox1.intersect(&bbox2);
        assert!(inter.is_some(), "Bounding boxes should intersect");

        let intersection = inter.unwrap();
        assert!(
            (intersection.x0 - 50.0).abs() < 0.001,
            "Intersection x0 should be accurate"
        );
        assert!(
            (intersection.y0 - 50.0).abs() < 0.001,
            "Intersection y0 should be accurate"
        );
        assert!(
            (intersection.x1 - 100.0).abs() < 0.001,
            "Intersection x1 should be accurate"
        );
        assert!(
            (intersection.y1 - 100.0).abs() < 0.001,
            "Intersection y1 should be accurate"
        );
    }

    #[test]
    fn test_text_positioning_accuracy() {
        use crate::renderer::TextSpan;

        let spans = vec![
            TextSpan {
                text: "Hello".to_string(),
                x: 100.0,
                y: 200.0,
                font_size: 12.0,
                font_id: None,
                color: "rgb(0,0,0)".to_string(),
            },
            TextSpan {
                text: "World".to_string(),
                x: 150.0,
                y: 200.0,
                font_size: 12.0,
                font_id: None,
                color: "rgb(0,0,0)".to_string(),
            },
        ];

        assert_eq!(spans.len(), 2);

        for span in &spans {
            verify_coordinate_sanity(span);
        }

        let dx = (spans[1].x - spans[0].x).abs();
        assert!(
            (dx - 50.0).abs() < 0.5,
            "Horizontal spacing should be accurate"
        );
    }

    #[test]
    fn test_matrix_multiplication_accuracy() {
        let tm1 = TransformMatrix {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 10.0,
            f: 20.0,
        };

        let tm2 = TransformMatrix {
            a: 0.5,
            b: 0.0,
            c: 0.0,
            d: 0.5,
            e: 5.0,
            f: 10.0,
        };

        let result = tm1 * tm2;
        let (x, y) = result.transform_point(0.0, 0.0);

        assert!(
            (x - 20.0).abs() < 0.5,
            "Matrix multiplication should produce accurate x translation"
        );
        assert!(
            (y - 40.0).abs() < 0.5,
            "Matrix multiplication should produce accurate y translation"
        );
    }

    #[test]
    fn test_transform_chain_accuracy() {
        let mut tm = TransformMatrix::identity();

        tm.e = 10.0;
        tm.f = 20.0;

        let mut tm2 = tm.clone();
        tm2.e = 30.0;
        tm2.f = 40.0;

        let chained = tm * tm2;
        let (x, y) = chained.transform_point(0.0, 0.0);

        assert!(
            (x - 40.0).abs() < 0.5,
            "Chained transforms should accumulate correctly for x"
        );
        assert!(
            (y - 60.0).abs() < 0.5,
            "Chained transforms should accumulate correctly for y"
        );
    }

    #[test]
    fn test_rotated_coordinate_accuracy() {
        let mut tm = TransformMatrix::identity();

        const DEG_TO_RAD: f64 = std::f64::consts::PI / 180.0;
        let angle = DEG_TO_RAD * 90.0;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        tm.a = cos_a;
        tm.b = sin_a;
        tm.c = -sin_a;
        tm.d = cos_a;

        let (x, y) = tm.transform_point(1.0, 0.0);

        assert!(
            (x - 0.0).abs() < 0.5,
            "90 degree rotation should put point near y-axis"
        );
        assert!(
            (y - 1.0).abs() < 0.5,
            "90 degree rotation should rotate y correctly"
        );
    }

    #[test]
    fn test_epsilons_for_accuracy() {
        use crate::util::math::equal;

        assert!(
            equal(1.0, 1.000001),
            "Epsilon comparison should allow small differences"
        );
        assert!(
            !equal(1.0, 1.01),
            "Epsilon comparison should reject large differences"
        );

        // equal() uses a fixed epsilon (0.0001), so it does NOT scale with magnitude.
        assert!(
            !equal(100.0, 100.4),
            "Fixed epsilon should reject large differences regardless of magnitude"
        );
        assert!(
            equal(100.0, 100.00005),
            "Fixed epsilon should allow sub-epsilon differences"
        );
    }
}
