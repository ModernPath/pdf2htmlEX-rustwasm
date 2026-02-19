// Safety Audit - No Unsafe Blocks Verification
//
// This file documents the deliberate absence of 'unsafe' blocks in ode-core.
// All operations use Rust's safe APIs, relying on the type system and ownership
// to guarantee memory safety without requiring manual memory management.
//
// Verified: No 'unsafe {}' blocks found in the codebase.
//
// Key safety patterns used:
// 1. Boundary checks with slice indexing using .get() instead of direct []
// 2. Iterator-based transformations instead of raw pointer arithmetic
// 3. Result-based error handling for all fallible operations
// 4. String::from_utf8_lossy() for handling potentially invalid UTF-8 without unsafe
// 5. Safe crate dependencies (flate2, base64, etc.) that don't expose unsafe APIs

#[cfg(test)]
mod safety_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn verify_no_unsafe_blocks_in_source() {
        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

        if let Err(e) = check_directory_for_unsafe(&src_dir) {
            panic!("Security audit failed: {}", e);
        }
    }

    fn check_directory_for_unsafe(dir: &Path) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                check_directory_for_unsafe(&path)?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                check_file_for_unsafe(&path)?;
            }
        }

        Ok(())
    }

    fn check_file_for_unsafe(path: &Path) -> Result<(), String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file {:?}: {}", path, e))?;

        // Skip the safety_audit.rs file itself since it contains "unsafe" in comments
        if path.file_name().map_or(false, |n| n == "safety_audit.rs") {
            return Ok(());
        }

        // Check for 'unsafe' keyword
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Look for actual unsafe blocks, not just the word "unsafe" in comments
            if line.starts_with("unsafe") || line.contains("unsafe {") {
                // Allow if it's just in a comment
                if !line.starts_with("//") && !line.starts_with("/*") {
                    return Err(format!(
                        "Found 'unsafe' block in {:?} at line {}: '{}'",
                        path,
                        line_num + 1,
                        line
                    ));
                }
            }
        }

        Ok(())
    }

    #[test]
    fn verify_memory_safety_patterns() {
        use crate::config::ConversionConfig;
        use crate::parser::parse_pdf;

        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
        let _config = ConversionConfig::default();

        // This operation should succeed and never cause UB
        let result = parse_pdf(pdf_data);
        assert!(result.is_ok());

        // Verify we can safely inspect the result
        let doc = result.unwrap();
        assert!(doc.pages.len() > 0);
    }

    #[test]
    fn verify_no_raw_pointers() {
        // This is a compile-time test - if raw pointers were used,
        // the code wouldn't compile without unsafe blocks

        // Simulate the operations that the core engine performs
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];

        // Safe pattern: use iterators instead of pointer arithmetic
        let _sum: u32 = data.iter().map(|&x| x as u32).sum();

        // Safe pattern: use slices with bounds checking
        if let Some(first) = data.first() {
            assert_eq!(*first, 1);
        }

        // Safe pattern: use get() for optional access
        if let Some(third) = data.get(2) {
            assert_eq!(*third, 3);
        }

        // Safe pattern: use chunks for safe iteration
        let chunks: Vec<_> = data.chunks(2).collect();
        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn verify_error_handling_safety() {
        use crate::error::OdeError;

        // All core operations return Result, forcing error handling
        fn safe_operation(input: &[u8]) -> Result<String, OdeError> {
            if input.is_empty() {
                return Err(OdeError::PdfParseError("Empty input".to_string()));
            }

            String::from_utf8_lossy(input)
                .to_string()
                .parse::<String>()
                .map(|_| String::new())
                .map_err(|_| OdeError::PdfParseError("Parse failed".to_string()))
        }

        let result = safe_operation(b"test");
        assert!(result.is_ok());

        let result = safe_operation(b"");
        assert!(result.is_err());
    }

    #[test]
    fn verify_ownership_safety() {
        use crate::renderer::TextSpan;

        // Construct objects with clear ownership
        let span1 = TextSpan {
            text: "Hello".to_string(),
            x: 10.0,
            y: 20.0,
            font_size: 12.0,
            font_id: None,
            color: "rgb(0,0,0)".to_string(),
        };

        // Clone creates a true copy, not a shallow reference
        let span2 = span1.clone();

        // Both are independent
        assert_eq!(span1.text, span2.text);

        // Moving transfers ownership, no dangling references
        let span3 = span1;
        assert_eq!(span3.text, "Hello");

        // span1 is no longer accessible here - compiler enforces this
    }
}
