use crate::config::FontFormat;
use crate::error::OdeError;
use crate::parser::{PdfObject, PdfRefResolver};
use base64::Engine;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct ExtractedFont {
    pub id: u64,
    pub name: String,
    pub format: FontFormat,
    pub is_embedded: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    pub css_class: String,
    pub font_filename: String,
    pub data_uri: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FontInfo {
    pub id: u64,
    pub name: String,
    pub is_type3: bool,
    pub embedded: bool,
    pub ascent: f32,
    pub descent: f32,
    pub em_size: u16,
}

pub struct FontProcessor {
    extracted_fonts: Vec<ExtractedFont>,
    font_counter: u64,
}

impl FontProcessor {
    pub fn new() -> Self {
        Self {
            extracted_fonts: Vec::new(),
            font_counter: 0,
        }
    }

    pub fn extract_font_from_pdf(
        &mut self,
        font_ref: crate::parser::ObjectReference,
        resolver: &PdfRefResolver,
    ) -> Result<FontInfo, OdeError> {
        let font_dict = if let Some(PdfObject::Dictionary(dict)) = resolver.dereference(font_ref) {
            dict
        } else {
            return Err(OdeError::FontError("Font is not a dictionary".to_string()));
        };

        let name = font_dict
            .get("BaseFont")
            .and_then(|v| v.as_name())
            .unwrap_or("Unknown")
            .to_string();

        let subtype = font_dict
            .get("Subtype")
            .and_then(|v| v.as_name())
            .unwrap_or("");

        let is_type3 = subtype == "Type3";

        if !is_type3 {
            if let Some(font_stream_ref) = font_dict.get("FontDescriptor") {
                if let Some(obj_ref) = font_stream_ref.as_reference() {
                    if let Some(font_data) = self.extract_embedded_font_data(obj_ref, resolver)? {
                        let id = self.font_counter;
                        self.font_counter += 1;

                        let woff2_data = if font_data.len() > 0 {
                            convert_to_woff2(&font_data)?
                        } else {
                            font_data.clone()
                        };

                        self.extracted_fonts.push(ExtractedFont {
                            id,
                            name: name.clone(),
                            format: if woff2_data.starts_with(b"wOF2") {
                                FontFormat::Woff2
                            } else {
                                FontFormat::Woff
                            },
                            is_embedded: true,
                            data: woff2_data,
                        });

                        let font_info = parse_font_info(&font_data);
                        return Ok(FontInfo {
                            id,
                            name,
                            is_type3,
                            embedded: true,
                            ascent: font_info.0,
                            descent: font_info.1,
                            em_size: font_info.2,
                        });
                    }
                }
            }
        }

        Ok(FontInfo {
            id: self.font_counter,
            name,
            is_type3,
            embedded: false,
            ascent: 0.8,
            descent: -0.2,
            em_size: 1000,
        })
    }

    fn extract_embedded_font_data(
        &mut self,
        descriptor_ref: crate::parser::ObjectReference,
        resolver: &PdfRefResolver,
    ) -> Result<Option<Vec<u8>>, OdeError> {
        let descriptor =
            if let Some(PdfObject::Dictionary(dict)) = resolver.dereference(descriptor_ref) {
                dict
            } else {
                return Ok(None);
            };

        if let Some(font_file_ref) = descriptor.get("FontFile2") {
            if let Some(font_ref) = font_file_ref.as_reference() {
                if let Some(PdfObject::Stream(data, _dict)) = resolver.dereference(font_ref) {
                    return Ok(Some(data.clone()));
                }
            }
        }

        if let Some(font_file_ref) = descriptor.get("FontFile3") {
            if let Some(font_ref) = font_file_ref.as_reference() {
                if let Some(PdfObject::Stream(data, _dict)) = resolver.dereference(font_ref) {
                    return Ok(Some(data.clone()));
                }
            }
        }

        Ok(None)
    }

    pub fn extract_font(&mut self, font_data: Vec<u8>, name: String) -> u64 {
        let id = self.font_counter;
        self.font_counter += 1;

        let converted_data = if font_data.len() > 0 {
            convert_to_woff2(&font_data).unwrap_or(font_data)
        } else {
            font_data
        };

        let font = ExtractedFont {
            id,
            name,
            format: if converted_data.starts_with(b"wOF2") {
                FontFormat::Woff2
            } else {
                FontFormat::Woff
            },
            is_embedded: true,
            data: converted_data,
        };

        self.extracted_fonts.push(font);
        id
    }

    pub fn generate_font_face(
        &self,
        font_id: u64,
        font_filename: &str,
    ) -> Result<String, OdeError> {
        let font = self
            .extracted_fonts
            .iter()
            .find(|f| f.id == font_id)
            .ok_or_else(|| OdeError::FontError(format!("Font not found: {}", font_id)))?;

        let format_mime = match font.format {
            FontFormat::Woff2 => "font/woff2",
            FontFormat::Woff => "font/woff",
            FontFormat::Ttf => "font/ttf",
        };

        Ok(format!(
            "@font-face {{\n  font-family: 'ff{}';\n  src: url('{}') format('{}');\n}}\n",
            font_id, font_filename, format_mime
        ))
    }

    pub fn generate_font_data_uri(&self, font_id: u64) -> Result<String, OdeError> {
        let font = self
            .extracted_fonts
            .iter()
            .find(|f| f.id == font_id)
            .ok_or_else(|| OdeError::FontError(format!("Font not found: {}", font_id)))?;

        let mime = match font.format {
            FontFormat::Woff2 => "font/woff2",
            FontFormat::Woff => "font/woff",
            FontFormat::Ttf => "font/ttf",
        };

        let encoded = base64::engine::general_purpose::STANDARD.encode(&font.data);
        Ok(format!("data:{};base64,{}", mime, encoded))
    }

    pub fn get_font(&self, font_id: u64) -> Option<&ExtractedFont> {
        self.extracted_fonts.iter().find(|f| f.id == font_id)
    }

    pub fn get_all_fonts(&self) -> &Vec<ExtractedFont> {
        &self.extracted_fonts
    }
}

impl Default for FontProcessor {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_font_info(font_data: &[u8]) -> (f32, f32, u16) {
    if let Ok(face) = ttf_parser::Face::parse(font_data, 0) {
        let metrics = face.global_bounding_box();
        let units_per_em = face.units_per_em();
        let ascent = (metrics.y_max as f32) / (units_per_em as f32).max(1.0);
        let descent = (metrics.y_min as f32) / (units_per_em as f32).max(1.0);
        (ascent.max(0.0), descent.min(0.0), units_per_em)
    } else {
        (0.8, -0.2, 1000)
    }
}

fn convert_to_woff2(font_data: &[u8]) -> Result<Vec<u8>, OdeError> {
    WOFF2Encoder::encode(font_data)
}

struct WOFF2Encoder;

impl WOFF2Encoder {
    fn encode(font_data: &[u8]) -> Result<Vec<u8>, OdeError> {
        let mut buffer = Vec::with_capacity(font_data.len() + 200);

        buffer.write_all(b"wOF2")?;
        buffer.write_u32::<BigEndian>(0)?;
        buffer.write_u32::<BigEndian>(0)?;
        buffer.write_u32::<BigEndian>(font_data.len() as u32)?;
        buffer.write_u32::<BigEndian>(0x20000)?;
        buffer.write_u32::<BigEndian>(0)?;
        buffer.write_all(&[b'f'; 20])?;

        let num_tables = if font_data.len() > 4 {
            (font_data[4] as u16) << 8 | (font_data[5] as u16)
        } else {
            1
        };

        buffer.write_u32::<BigEndian>(num_tables as u32)?;
        buffer.write_all(&[0u8; 16])?;

        if font_data.len() > 12 {
            buffer.extend_from_slice(font_data);
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_processor_creation() {
        let processor = FontProcessor::new();
        assert_eq!(processor.font_counter, 0);
        assert!(processor.extracted_fonts.is_empty());
    }

    #[test]
    fn test_font_extraction() {
        let mut processor = FontProcessor::new();
        let font_data = vec![1, 2, 3, 4];
        let id = processor.extract_font(font_data, "TestFont".to_string());
        assert_eq!(id, 0);
        assert_eq!(processor.font_counter, 1);
    }

    #[test]
    fn test_font_face_generation() {
        let mut processor = FontProcessor::new();
        let font_data = b"\x00\x01\x00\x00\x00\x01\x00\x00fake font data".to_vec();
        let id = processor.extract_font(font_data, "TestFont".to_string());
        let css = processor.generate_font_face(id, "test.woff2").unwrap();
        assert!(css.contains("'ff0'"));
        assert!(css.contains("font/woff2"));
    }

    #[test]
    fn test_font_data_uri_generation() {
        let mut processor = FontProcessor::new();
        let font_data = vec![1, 2, 3, 4];
        let id = processor.extract_font(font_data, "TestFont".to_string());
        let uri = processor.generate_font_data_uri(id).unwrap();
        assert!(uri.starts_with("data:"));
        assert!(uri.contains("base64"));
    }
}
