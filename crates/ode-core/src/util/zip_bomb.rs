use crate::error::{OdeError, OdeResult};
use flate2::read::{DeflateDecoder, ZlibDecoder};
use std::io::{Cursor, Read};

pub struct ZipBombDetector {
    max_ratio: u32,
}

impl ZipBombDetector {
    pub fn new(max_ratio: u32) -> Self {
        Self { max_ratio }
    }

    pub fn with_limit_100_to_1() -> Self {
        Self::new(100)
    }

    pub fn check_compressed_data(
        &self,
        compressed: &[u8],
        method: CompressionMethod,
    ) -> OdeResult<usize> {
        if compressed.is_empty() {
            return Err(OdeError::PdfParseError("Empty compressed data".to_string()));
        }

        let decompressed_size = self.measure_decompressed_size(compressed, method)?;

        let ratio = decompressed_size as u32 / compressed.len() as u32;

        if ratio > self.max_ratio {
            return Err(OdeError::ZipBomb { ratio });
        }

        Ok(decompressed_size)
    }

    fn measure_decompressed_size(
        &self,
        compressed: &[u8],
        method: CompressionMethod,
    ) -> OdeResult<usize> {
        const MAX_DECOMPRESSED: usize = 10 * 1024 * 1024; // 10MB limit for measurement

        let mut buffer = Vec::with_capacity(1024);
        let mut cursor = Cursor::new(compressed);
        let mut decoder: Box<dyn Read> = match method {
            CompressionMethod::Flate => Box::new(DeflateDecoder::new(&mut cursor)),
            CompressionMethod::FlateDecode => Box::new(ZlibDecoder::new(&mut cursor)),
            CompressionMethod::DCTDecode => {
                return Err(OdeError::Unsupported(
                    "DCTDecode (JPEG) compression not supported for zip bomb detection".to_string(),
                ));
            }
            CompressionMethod::CCITTFaxDecode => {
                return Err(OdeError::Unsupported(
                    "CCITTFaxDecode compression not supported for zip bomb detection".to_string(),
                ));
            }
            CompressionMethod::ASCII85Decode => {
                return Ok(self.measure_ascii85(compressed)?);
            }
            CompressionMethod::ASCIIHexDecode => {
                return Ok(self.measure_asciihex(compressed)?);
            }
            CompressionMethod::LZWDecode => {
                return Ok(compressed.len() * 2);
            }
            CompressionMethod::RunLengthDecode => {
                return Ok(compressed.len() * 4);
            }
        };

        let mut temp_buf = [0u8; 4096];
        loop {
            let bytes_read = decoder
                .read(&mut temp_buf)
                .map_err(|e| OdeError::PdfParseError(format!("Decompression error: {}", e)))?;

            if bytes_read == 0 {
                break;
            }

            buffer.extend_from_slice(&temp_buf[..bytes_read]);

            if buffer.len() > MAX_DECOMPRESSED {
                return Err(OdeError::ZipBomb { ratio: u32::MAX });
            }
        }

        Ok(buffer.len())
    }

    fn measure_ascii85(&self, compressed: &[u8]) -> OdeResult<usize> {
        let mut estimated_size = 0;
        let mut i = 0;

        while i < compressed.len() {
            let byte = compressed[i];

            if byte.is_ascii_whitespace() {
                i += 1;
                continue;
            }

            if byte == b'z' || byte == b'Z' {
                estimated_size += 4;
                i += 1;
            } else if byte >= 33 && byte <= 117 {
                estimated_size += 4;
                i += 5;
            } else {
                i += 1;
            }

            if estimated_size > MAX_DECOMPRESSED_SIZE {
                return Err(OdeError::ZipBomb { ratio: u32::MAX });
            }
        }

        Ok(estimated_size)
    }

    fn measure_asciihex(&self, compressed: &[u8]) -> OdeResult<usize> {
        let mut estimated_size = 0;
        let mut i = 0;

        while i < compressed.len() {
            let byte = compressed[i];

            if byte.is_ascii_whitespace() {
                i += 1;
                continue;
            }

            if byte == b'>' {
                break;
            }

            if byte.is_ascii_hexdigit() {
                estimated_size += 1;
            }
            i += 1;

            if estimated_size > MAX_DECOMPRESSED_SIZE {
                return Err(OdeError::ZipBomb { ratio: u32::MAX });
            }
        }

        Ok(estimated_size / 2)
    }

    pub fn check_buffer(&self, data: &[u8], suspected_compressed: bool) -> OdeResult<()> {
        if !suspected_compressed || data.len() < 2 {
            return Ok(());
        }

        // A compressed buffer of this size could decompress to at most
        // data.len() * max_ratio bytes. If that exceeds our hard limit,
        // we reject early as a potential zip bomb. Use u64 to avoid overflow.
        let max_decompressed = data.len() as u64 * self.max_ratio as u64;
        if max_decompressed > MAX_DECOMPRESSED_SIZE as u64 {
            return Err(OdeError::ZipBomb {
                ratio: self.max_ratio,
            });
        }

        Ok(())
    }
}

const MAX_DECOMPRESSED_SIZE: usize = 100 * 1024 * 1024; // 100MB hard limit

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionMethod {
    Flate,
    FlateDecode,
    DCTDecode,
    CCITTFaxDecode,
    ASCII85Decode,
    ASCIIHexDecode,
    LZWDecode,
    RunLengthDecode,
}

impl Default for ZipBombDetector {
    fn default() -> Self {
        Self::with_limit_100_to_1()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = ZipBombDetector::new(50);
        assert_eq!(detector.max_ratio, 50);
    }

    #[test]
    fn test_default_detector() {
        let detector = ZipBombDetector::default();
        assert_eq!(detector.max_ratio, 100);
    }

    #[test]
    fn test_safe_compression() {
        let detector = ZipBombDetector::with_limit_100_to_1();
        let compressed = vec![0, 1, 2, 3, 4];
        let result = detector.check_compressed_data(&compressed, CompressionMethod::ASCII85Decode);
        assert!(result.is_ok() || matches!(result.unwrap_err(), OdeError::PdfParseError(_)));
    }

    #[test]
    fn test_empty_data() {
        let detector = ZipBombDetector::default();
        let result = detector.check_compressed_data(&[], CompressionMethod::Flate);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_buffer_safe() {
        let detector = ZipBombDetector::default();
        let data = vec![1, 2, 3, 4, 5];
        let result = detector.check_buffer(&data, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_buffer_large() {
        let detector = ZipBombDetector::default();
        let large_data = vec![0u8; 10_000_001];
        let result = detector.check_buffer(&large_data, true);
        assert!(matches!(result, Err(OdeError::ZipBomb { .. })));
    }

    #[test]
    fn test_ascii85_measurement() {
        let detector = ZipBombDetector::default();
        let ascii85 = b"Hello";
        let result = detector.measure_ascii85(ascii85);
        assert!(result.is_ok());
    }

    #[test]
    fn test_asciihex_measurement() {
        let detector = ZipBombDetector::default();
        let asciihex = b"48656C6C6F>";
        let result = detector.measure_asciihex(asciihex);
        assert!(result.is_ok());
    }
}
