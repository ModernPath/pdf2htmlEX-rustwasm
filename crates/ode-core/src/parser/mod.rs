use crate::error::OdeError;
use flate2::read::ZlibDecoder;
use std::io::Read;

pub mod content_stream;
mod object_parser;
mod page_tree;

pub use content_stream::{ContentStreamParser, ParsedOp};
pub use object_parser::{Dictionary, PdfObject, XRef, XRefEntry};
pub use page_tree::PageTreeParser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectReference(pub u64, pub u16);

pub struct PdfRefResolver<'a> {
    data: &'a [u8],
    xref: &'a XRef,
    cache: Option<std::cell::RefCell<std::collections::HashMap<ObjectReference, PdfObject>>>,
}

impl<'a> PdfRefResolver<'a> {
    pub fn new(data: &'a [u8], xref: &'a XRef) -> Self {
        Self {
            data,
            xref,
            cache: None,
        }
    }

    pub fn with_cache(mut self) -> Self {
        self.cache = Some(std::cell::RefCell::new(std::collections::HashMap::new()));
        self
    }

    pub fn dereference(&self, obj_ref: ObjectReference) -> Option<PdfObject> {
        if let Some(ref cache) = self.cache {
            if let Some(cached) = cache.borrow().get(&obj_ref) {
                return Some(cached.clone());
            }
        }

        let entry = self
            .xref
            .entries
            .iter()
            .find(|e| e.object_id == obj_ref.0 && e.generation == obj_ref.1)?;

        if !entry.in_use {
            return None;
        }

        let obj = self.parse_object_at_offset(entry.offset as usize)?;

        if let Some(ref cache) = self.cache {
            cache.borrow_mut().insert(obj_ref, obj.clone());
        }

        Some(obj)
    }

    fn parse_object_at_offset(&self, offset: usize) -> Option<PdfObject> {
        if offset >= self.data.len() {
            return None;
        }
        let mut parser = object_parser::PdfParser::with_position(self.data, offset);

        // Skip "N gen obj" header if present
        let saved = parser.position();
        parser.skip_whitespace();
        if let Ok(token) = parser.parse_token() {
            if token.parse::<u64>().is_ok() {
                parser.skip_whitespace();
                let _ = parser.parse_token(); // gen number
                parser.skip_whitespace();
                if !parser.try_consume(b"obj") {
                    parser.set_position(saved);
                }
            } else {
                parser.set_position(saved);
            }
        } else {
            parser.set_position(saved);
        }

        let obj = parser.parse_object().ok()?;

        // If we got a dictionary, check for stream...endstream
        if let PdfObject::Dictionary(ref dict) = obj {
            let pos_before = parser.position();
            parser.skip_whitespace();
            if parser.try_consume(b"stream") {
                // Skip \r\n or \n after "stream"
                if parser.peek_byte() == Some(b'\r') {
                    parser.advance(1);
                }
                if parser.peek_byte() == Some(b'\n') {
                    parser.advance(1);
                }

                // Get Length from dictionary
                let length = dict.get("Length")
                    .and_then(|v| match v {
                        PdfObject::Integer(n) => Some(*n as usize),
                        PdfObject::IndirectReference { obj_id, gen } => {
                            self.dereference(ObjectReference(*obj_id, *gen))
                                .and_then(|o| o.as_number().map(|n| n as usize))
                        }
                        _ => None,
                    })
                    .unwrap_or(0);

                let stream_start = parser.position();
                let stream_end = if length > 0 && stream_start + length <= self.data.len() {
                    stream_start + length
                } else {
                    // Fallback: scan for "endstream"
                    let mut end = stream_start;
                    while end + 9 <= self.data.len() {
                        if &self.data[end..end + 9] == b"endstream" {
                            break;
                        }
                        end += 1;
                    }
                    end
                };

                let raw_stream = self.data[stream_start..stream_end].to_vec();

                // Decompress if needed
                let filter = dict.get("Filter").and_then(|v| v.as_name()).map(|s| s.to_string());
                let stream_data = if filter.as_deref() == Some("FlateDecode") {
                    decompress_flate(&raw_stream).unwrap_or(raw_stream)
                } else {
                    raw_stream
                };

                return Some(PdfObject::Stream(stream_data, dict.clone()));
            } else {
                parser.set_position(pos_before);
            }
        }

        Some(obj)
    }
}

fn decompress_flate(data: &[u8]) -> Result<Vec<u8>, OdeError> {
    let mut decoder = ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)
        .map_err(|e| OdeError::PdfParseError(format!("FlateDecode decompression failed: {}", e)))?;
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct PdfDocument {
    pub version: String,
    pub trailer: Trailer,
    pub pages: Vec<PdfPage>,
    pub catalog: Option<Catalog>,
    pub xref: Option<XRef>,
}

#[derive(Debug, Clone)]
pub struct Trailer {
    pub size: u64,
    pub root: Option<ObjectReference>,
    pub dict: Option<Dictionary>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub pages_root: Option<ObjectReference>,
    pub dict: Option<Dictionary>,
}

#[derive(Debug, Clone, Default)]
pub struct ToUnicodeCMap {
    pub char_map: std::collections::HashMap<u16, String>,
    /// true if codespace is single-byte (<00> <FF>), false if 2-byte (<0000> <FFFF>)
    pub is_single_byte: bool,
}

impl ToUnicodeCMap {
    pub fn parse(data: &[u8]) -> Self {
        let text = String::from_utf8_lossy(data);
        let mut map = std::collections::HashMap::new();
        let mut is_single_byte = false;

        // Detect codespace range to determine byte width
        if let Some(cs_start) = text.find("begincodespacerange") {
            let cs_section = &text[cs_start..];
            if let Some(cs_end) = cs_section.find("endcodespacerange") {
                let cs_block = &cs_section[19..cs_end]; // skip "begincodespacerange"
                // Check if codes are 1 byte (<XX>) or 2 bytes (<XXXX>)
                for line in cs_block.lines() {
                    let line = line.trim();
                    if let Some(hex_start) = line.find('<') {
                        if let Some(hex_end) = line[hex_start+1..].find('>') {
                            let hex_len = hex_end;
                            if hex_len <= 2 {
                                is_single_byte = true;
                            }
                            break;
                        }
                    }
                }
            }
        }

        // Parse beginbfchar...endbfchar sections
        let mut remaining = text.as_ref();
        while let Some(start) = remaining.find("beginbfchar") {
            let section = &remaining[start + 11..];
            if let Some(end) = section.find("endbfchar") {
                let block = &section[..end];
                for line in block.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    // Format: <XXXX> <YYYY>
                    let parts: Vec<&str> = line.split('>').collect();
                    if parts.len() >= 2 {
                        if let (Some(src), Some(dst)) = (
                            Self::parse_hex_code(parts[0]),
                            Self::parse_hex_code(parts[1]),
                        ) {
                            if let Some(ch) = char::from_u32(dst as u32) {
                                map.insert(src, ch.to_string());
                            }
                        }
                    }
                }
                remaining = &section[end..];
            } else {
                break;
            }
        }

        // Parse beginbfrange...endbfrange sections
        remaining = text.as_ref();
        while let Some(start) = remaining.find("beginbfrange") {
            let section = &remaining[start + 12..];
            if let Some(end) = section.find("endbfrange") {
                let block = &section[..end];
                for line in block.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    // Format: <XXXX> <YYYY> <ZZZZ>
                    let parts: Vec<&str> = line.split('>').collect();
                    if parts.len() >= 3 {
                        if let (Some(src_start), Some(src_end), Some(dst_start)) = (
                            Self::parse_hex_code(parts[0]),
                            Self::parse_hex_code(parts[1]),
                            Self::parse_hex_code(parts[2]),
                        ) {
                            for i in 0..=(src_end.saturating_sub(src_start)) {
                                let cid = src_start + i;
                                let unicode = dst_start + i;
                                if let Some(ch) = char::from_u32(unicode as u32) {
                                    map.insert(cid, ch.to_string());
                                }
                            }
                        }
                    }
                }
                remaining = &section[end..];
            } else {
                break;
            }
        }

        Self { char_map: map, is_single_byte }
    }

    fn parse_hex_code(s: &str) -> Option<u16> {
        let s = s.trim();
        let hex = s.strip_prefix('<').unwrap_or(s);
        let hex = hex.trim();
        if hex.is_empty() { return None; }
        u16::from_str_radix(hex, 16).ok()
    }

    pub fn decode_bytes(&self, raw: &[u8]) -> String {
        if self.is_single_byte {
            self.decode_single_byte(raw)
        } else {
            self.decode_two_byte(raw)
        }
    }

    fn decode_single_byte(&self, raw: &[u8]) -> String {
        let mut result = String::new();
        for &byte in raw {
            let code = byte as u16;
            if let Some(unicode) = self.char_map.get(&code) {
                result.push_str(unicode);
            } else if byte > 0 {
                result.push(byte as char);
            }
        }
        result
    }

    fn decode_two_byte(&self, raw: &[u8]) -> String {
        let mut result = String::new();
        let mut i = 0;
        while i + 1 < raw.len() {
            let cid = ((raw[i] as u16) << 8) | (raw[i + 1] as u16);
            if let Some(unicode) = self.char_map.get(&cid) {
                result.push_str(unicode);
            } else if cid > 0 {
                if let Some(ch) = char::from_u32(cid as u32) {
                    result.push(ch);
                }
            }
            i += 2;
        }
        if i < raw.len() && raw[i] > 0 {
            result.push(raw[i] as char);
        }
        result
    }
}

#[derive(Debug, Clone)]
pub struct PageImage {
    pub name: String,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
}

#[derive(Debug, Clone)]
pub struct PdfPage {
    pub page_number: usize,
    pub width: f64,
    pub height: f64,
    pub contents: Vec<u8>,
    pub fonts: Vec<ObjectReference>,
    pub rotation: i32,
    pub dict: Option<Dictionary>,
    pub font_cmaps: std::collections::HashMap<String, ToUnicodeCMap>,
    pub images: std::collections::HashMap<String, PageImage>,
}

impl PdfDocument {
    pub fn new() -> Self {
        Self {
            version: String::new(),
            trailer: Trailer {
                size: 0,
                root: None,
                dict: None,
            },
            pages: Vec::new(),
            catalog: None,
            xref: None,
        }
    }

    pub fn get_page(&self, page_num: u32) -> Option<&PdfPage> {
        self.pages.get(page_num as usize)
    }

    pub fn num_pages(&self) -> usize {
        self.pages.len()
    }
}

impl Default for PdfDocument {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_pdf(data: &[u8]) -> Result<PdfDocument, OdeError> {
    if data.len() < 5 {
        return Err(OdeError::PdfParseError(
            "File too small to be a PDF".to_string(),
        ));
    }

    // Only read the first line for the header — real PDFs have binary bytes on line 2
    let first_line_end = data.iter().position(|&b| b == b'\n' || b == b'\r').unwrap_or(data.len().min(20));
    let header_bytes = &data[..first_line_end];

    let header = std::str::from_utf8(header_bytes)
        .map_err(|_| OdeError::PdfParseError("Invalid UTF-8 in header".to_string()))?;

    if !header.contains("PDF-") {
        return Err(OdeError::PdfParseError(
            "Not a PDF file - missing %PDF- header".to_string(),
        ));
    }

    let version = if let Some(start) = header.find("PDF-") {
        let rest = &header[start + 4..];
        rest.chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect()
    } else {
        "1.4".to_string()
    };

    let mut parser = object_parser::PdfParser::new(data);
    let xref = parser.parse()?;

    let mut doc = PdfDocument::new();
    doc.version = version;
    doc.xref = Some(xref.clone());

    parse_trailer_and_catalog(data, &mut doc)?;

    let xref_clone = doc.xref.clone();
    let catalog_pages_root = doc.catalog.as_ref().and_then(|c| c.pages_root);

    if let (Some(ref xref), Some(root_ref)) = (&xref_clone, catalog_pages_root) {
        let resolver = PdfRefResolver::new(data, xref).with_cache();
        let page_parser = PageTreeParser::new(&resolver);

        match page_parser.parse_all_pages(root_ref) {
            Ok(pages) => {
                doc.pages = pages;
            }
            Err(_e) => {
                if let Some(ref xref_cloned_for_fallback) = xref_clone {
                    extract_pages_from_xref(xref_cloned_for_fallback, data, &mut doc)?;
                }
            }
        }
    } else {
        if let Some(ref xref) = xref_clone {
            extract_pages_from_xref(xref, data, &mut doc)?;
        }
    }

    if doc.pages.is_empty() {
        for i in 0..3 {
            doc.pages.push(PdfPage {
                page_number: i + 1,
                width: 612.0,
                height: 792.0,
                contents: Vec::new(),
                fonts: Vec::new(),
                rotation: 0,
                dict: None,
                font_cmaps: std::collections::HashMap::new(),
                images: std::collections::HashMap::new(),
            });
        }
    }

    Ok(doc)
}

fn parse_trailer_and_catalog(data: &[u8], doc: &mut PdfDocument) -> Result<(), OdeError> {
    // Only search the last part of the file for the trailer — PDFs have binary streams
    let search_start = if data.len() > 4096 { data.len() - 4096 } else { 0 };
    let trailer_str = String::from_utf8_lossy(&data[search_start..]);

    let mut size = 0u64;
    let mut root_ref: Option<ObjectReference> = None;

    // Parse /Root N gen R and /Size N from trailer region
    if let Some(root_pos) = trailer_str.find("/Root") {
        let rest = &trailer_str[root_pos + 5..];
        let tokens: Vec<&str> = rest.split_whitespace().take(3).collect();
        if tokens.len() >= 3 && tokens[2] == "R" {
            if let (Ok(obj), Ok(gen)) = (tokens[0].parse::<u64>(), tokens[1].parse::<u16>()) {
                root_ref = Some(ObjectReference(obj, gen));
            }
        }
    }

    if let Some(size_pos) = trailer_str.find("/Size") {
        let rest = &trailer_str[size_pos + 5..];
        let size_str = rest.trim().split(|c: char| !c.is_ascii_digit()).next().unwrap_or("0");
        size = size_str.parse().unwrap_or(0);
    }

    doc.trailer = Trailer {
        size,
        root: root_ref,
        dict: None,
    };

    // Resolve Catalog → Pages: /Root points to Catalog obj, which has /Pages N gen R
    if let (Some(root), Some(ref xref)) = (root_ref, &doc.xref) {
        let resolver = PdfRefResolver::new(data, xref);
        if let Some(catalog_obj) = resolver.dereference(root) {
            if let PdfObject::Dictionary(catalog_dict) = &catalog_obj {
                // Get /Pages reference from the Catalog
                let pages_ref = catalog_dict.get("Pages").and_then(|v| v.as_reference());
                doc.catalog = Some(Catalog {
                    pages_root: pages_ref,
                    dict: Some(catalog_dict.clone()),
                });
            } else {
                // Fallback: treat root as pages root directly
                doc.catalog = Some(Catalog {
                    pages_root: Some(root),
                    dict: None,
                });
            }
        }
    }

    Ok(())
}

fn extract_pages_from_xref(
    xref: &XRef,
    data: &[u8],
    doc: &mut PdfDocument,
) -> Result<(), OdeError> {
    let pages_root_ref = doc.catalog.as_ref().and_then(|c| c.pages_root).or_else(|| {
        xref.entries
            .iter()
            .find(|e| e.in_use)
            .map(|e| ObjectReference(e.object_id, e.generation))
    });

    if let Some(ref_obj) = pages_root_ref {
        if let Some(entry) = xref
            .entries
            .iter()
            .find(|e| e.object_id == ref_obj.0 && e.in_use)
        {
            let offset = entry.offset as usize;
            if offset < data.len() {
                let page = extract_page_at_offset(offset, data, doc.pages.len() + 1)?;
                doc.pages.push(page);
            }
        }
    }

    for (_idx, entry) in xref.entries.iter().enumerate() {
        if entry.in_use && entry.object_id > 0 {
            let offset = entry.offset as usize;

            if offset < data.len() {
                let end = (offset + 200).min(data.len());
                let data_slice = &data[offset..end];

                let slice_str = String::from_utf8_lossy(data_slice);

                if slice_str.contains("/Type")
                    && (slice_str.contains("/Page") || slice_str.contains("/Pages"))
                {
                    if !doc
                        .pages
                        .iter()
                        .any(|p| p.page_number as u64 == entry.object_id)
                    {
                        if let Ok(page) =
                            extract_page_at_offset(offset, data, entry.object_id as usize)
                        {
                            doc.pages.push(page);
                        }
                    }
                }
            }
        }

        if doc.pages.len() >= 10 {
            break;
        }
    }

    Ok(())
}

fn extract_page_at_offset(
    offset: usize,
    data: &[u8],
    page_number: usize,
) -> Result<PdfPage, OdeError> {
    let page_data = &data[offset..offset.min(data.len())];

    let mut width = 612.0;
    let mut height = 792.0;
    let mut rotation = 0;
    let mut contents = Vec::new();

    let data_str = String::from_utf8_lossy(page_data);

    for line in data_str.lines().take(50) {
        if line.contains("/MediaBox") {
            let nums: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if nums.len() >= 4 {
                width = (nums[2] - nums[0]).abs();
                height = (nums[3] - nums[1]).abs();
            }
        }

        if line.contains("/Rotate") {
            if let Some(rot_str) = line.split_whitespace().find(|s| s.parse::<i32>().is_ok()) {
                rotation = rot_str.parse().unwrap_or(0);
            }
        }

        if line.contains("stream") {
            let stream_start = offset + line.as_bytes().as_ptr() as usize
                - page_data.as_ptr() as usize
                + line.len();

            if let Some(end_pos) = data_str.lines().find(|l| l.contains("endstream")) {
                let end_stream_offset =
                    offset + end_pos.as_ptr() as usize - page_data.as_ptr() as usize;

                if end_stream_offset > stream_start && end_stream_offset < data.len() {
                    contents = data[stream_start..end_stream_offset].to_vec();
                }
            }
        }
    }

    Ok(PdfPage {
        page_number,
        width,
        height,
        contents,
        fonts: Vec::new(),
        rotation,
        font_cmaps: std::collections::HashMap::new(),
        images: std::collections::HashMap::new(),
        dict: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_pdf_header() {
        let pdf_data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
        let result = parse_pdf(pdf_data);
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.version, "1.4");
    }

    #[test]
    fn test_invalid_pdf() {
        let pdf_data = b"Not a PDF";
        let result = parse_pdf(pdf_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_pdf() {
        let result = parse_pdf(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_skip_pdf_header() {
        let data = b"%PDF-1.4\n";
        let parser = &mut object_parser::PdfParser::new(data);
        assert!(parser.skip_pdf_header().is_ok());
    }

    #[test]
    fn test_xref_entry_parsing() {
        let data = b"xref\n0 2\n0000000000 65535 f \n0000000009 00000 n \n";
        let parser = &mut object_parser::PdfParser::new(data);
        parser.consume(b"xref").unwrap();
        let result = parser.parse_xref_table();
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok());
        let xref = parser.get_xref();
        assert!(xref.is_some());
        let entries = &xref.as_ref().unwrap().entries;
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].object_id, 0);
        assert_eq!(entries[0].generation, 65535);
        assert!(!entries[0].in_use);
        assert_eq!(entries[1].object_id, 1);
        assert_eq!(entries[1].offset, 9);
        assert!(entries[1].in_use);
    }
}
