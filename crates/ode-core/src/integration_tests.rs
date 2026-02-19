use crate::config::ConversionConfig;
use crate::error::OdeError;
use crate::util::{TimeoutWrapper, ZipBombDetector};
use std::thread;
use std::time::Duration;

#[test]
fn test_zip_bomb_detector_rejects_high_compression() {
    let detector = ZipBombDetector::with_limit_100_to_1();

    let compressed = vec
![0u8; 1000];
    detector.check_buffer(&compressed, true)
.unwrap();

    let large_compressed = vec
![0u8; 11_000_000];
    let result = detector.check_buffer(&large_compressed, true);
    assert!(matches!(result, Err(OdeError::ZipBomb { .. })));
}

#[test]
fn test_timeout_wrapper_basic() {
    let wrapper = TimeoutWrapper::new(100);
    let result = wrapper.run_sync(|| Ok::<_, OdeError>("success".to_string()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

#[test]
fn test_timeout_wrapper_sync_behavior() {
    let wrapper = TimeoutWrapper::new(1000);

    let value = 42;
    let result = wrapper.run_sync(|| Ok::<_, OdeError>(value + 1));

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 43);
}

#[test]
fn test_conversion_config_defaults() {
    let config = ConversionConfig::default();

    assert_eq!(config.desired_dpi, 72.0);
    assert_eq!(config.font_format, crate::config::FontFormat::Woff2);
    assert_eq!(config.page_range, (1, usize::MAX));
    assert!(config.embed_font);
    assert!(config.correct_text_visibility);
}

#[test]
fn test_timeout_value_from_config() {
    let config = ConversionConfig {
        timeout_ms: Some(5000),
        ..Default::default()
    };

    assert_eq!(config.timeout_ms, Some(5000));
}

#[test]
fn test_zip_bomb_detector_custom_ratio() {
    let detector = ZipBombDetector::new(50);
    let _ = detector;
}

#[test]
fn test_zip_bomb_detector_with_limit() {
    let detector = ZipBombDetector::with_limit_100_to_1();
    let _ = detector;
}

#[test]
fn test_conversion_with_timeout() {
    let wrapper = TimeoutWrapper::new(100);

    let result = wrapper.run_sync(|| {
        thread::sleep(Duration::from_millis(10));
        Ok::<_, OdeError>("completed".to_string())
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "completed");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_timeout_wrapper_async() {
    let wrapper = TimeoutWrapper::new(100);

    let result = wrapper.run(|| async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok::<_, OdeError>("async completed".to_string())
    })
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "async completed");
}

#[test]
fn test_error_types() {
    let pdf_error = OdeError::PdfParseError("test error".to_string());
    assert!(pdf_error.to_string().contains("PDF parsing error"));

    let timeout_error = OdeError::Timeout("operation timed out".to_string());
    assert!(timeout_error.to_string().contains("Timeout"));

    let zip_bomb_error = OdeError::ZipBomb { ratio: 200 };
    assert!(zip_bomb_error.to_string().contains("zip bomb") && zip_bomb_error.to_string().contains("200"));
}