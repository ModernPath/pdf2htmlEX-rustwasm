use crate::error::OdeError;
use flate2::read::ZlibDecoder;
use std::io::Read;

#[derive(Debug, Clone, Default)]
pub struct XRef {
    pub entries: Vec<XRefEntry>,
    pub trailer: Option<Dictionary>,
}

#[derive(Debug, Clone, Copy)]
pub struct XRefEntry {
    pub object_id: u64,
    pub generation: u16,
    pub offset: u64,
    pub in_use: bool,
    /// For type-2 (compressed) entries: object stream number containing this object
    pub objstm_num: Option<u64>,
    /// For type-2 entries: index within the object stream
    pub objstm_idx: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Dictionary {
    pub entries: Vec<(String, PdfObject)>,
}

impl Dictionary {
    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub fn entries(&self) -> &Vec<(String, PdfObject)> {
        &self.entries
    }
}

#[derive(Debug, Clone)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Name(String),
    Array(Vec<PdfObject>),
    Dictionary(Dictionary),
    Stream(Vec<u8>, Dictionary),
    IndirectReference { obj_id: u64, gen: u16 },
}

impl PdfObject {
    pub fn as_reference(&self) -> Option<crate::parser::ObjectReference> {
        match self {
            PdfObject::IndirectReference { obj_id, gen } => {
                Some(crate::parser::ObjectReference(*obj_id, *gen))
            }
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            PdfObject::Name(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            PdfObject::Integer(n) => Some(*n as f64),
            PdfObject::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<PdfObject>> {
        match self {
            PdfObject::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            PdfObject::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Extract raw bytes from a String object. Converts each char back to u8.
    /// Works for hex strings and literal strings with byte values 0-255.
    pub fn as_raw_bytes(&self) -> Option<Vec<u8>> {
        match self {
            PdfObject::String(s) => Some(s.chars().map(|c| c as u8).collect()),
            _ => None,
        }
    }

    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PdfObject::Integer(n) => Some(*n),
            _ => None,
        }
    }
}

pub struct PdfParser<'a> {
    data: &'a [u8],
    pos: usize,
    xref: Option<XRef>,
}

impl<'a> PdfParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            xref: None,
        }
    }

    pub fn with_position(data: &'a [u8], pos: usize) -> Self {
        Self {
            data,
            pos,
            xref: None,
        }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn set_position(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    pub fn parse(&mut self) -> Result<XRef, OdeError> {
        self.skip_pdf_header()?;
        self.find_and_parse_xref()?;
        Ok(self.xref.clone().unwrap_or_default())
    }

    pub fn get_xref(&self) -> Option<&XRef> {
        self.xref.as_ref()
    }

    pub fn skip_pdf_header(&mut self) -> Result<(), OdeError> {
        // Only check the first line — real PDFs have binary bytes on line 2
        let first_line_end = self.data.iter().position(|&b| b == b'\n' || b == b'\r').unwrap_or(self.data.len().min(20));
        let header = std::str::from_utf8(&self.data[..first_line_end])
            .map_err(|_| OdeError::PdfParseError("Invalid UTF-8 in header".to_string()))?;

        if !header.contains("PDF-") {
            return Err(OdeError::PdfParseError("Not a PDF file".to_string()));
        }

        self.pos = first_line_end;
        while self.pos < self.data.len()
            && self.data[self.pos] != b'\n'
            && self.data[self.pos] != b'\r'
        {
            self.pos += 1;
        }
        while self.pos < self.data.len()
            && (self.data[self.pos] == b'\n' || self.data[self.pos] == b'\r')
        {
            self.pos += 1;
        }

        Ok(())
    }

    fn find_and_parse_xref(&mut self) -> Result<(), OdeError> {
        if let Some(xref_pos) = self.find_xref_offset() {
            self.parse_xref_at(xref_pos)?;
            // Follow /Prev chain to merge earlier xref sections
            self.follow_prev_chain()?;
        }

        Ok(())
    }

    fn find_xref_offset(&self) -> Option<usize> {
        // Only search the last part of the file — PDFs have binary streams
        let search_start = if self.data.len() > 4096 { self.data.len() - 4096 } else { 0 };
        let tail = String::from_utf8_lossy(&self.data[search_start..]);
        let eof_pos = tail.rfind("%%EOF")?;
        let startxref_pos = tail[..eof_pos].rfind("startxref")?;

        let offset_str = tail[startxref_pos + 9..eof_pos]
            .trim()
            .split_whitespace()
            .next()?;
        offset_str.parse::<usize>().ok()
    }

    fn parse_xref_at(&mut self, offset: usize) -> Result<(), OdeError> {
        self.pos = offset;
        self.skip_whitespace();

        if self.try_consume(b"xref") {
            self.parse_xref_table()?;
        } else if self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            // XRef stream object: "N gen obj << /Type /XRef ... >> stream ... endstream"
            self.parse_xref_stream_object()?;
        } else if self.try_consume(b"trailer") {
            self.parse_trailer_only()?;
        } else {
            return Err(OdeError::PdfParseError(
                format!("Invalid xref section at offset {}", offset),
            ));
        }

        Ok(())
    }

    fn follow_prev_chain(&mut self) -> Result<(), OdeError> {
        let mut visited = std::collections::HashSet::new();
        loop {
            let prev_offset = self.xref.as_ref()
                .and_then(|x| x.trailer.as_ref())
                .and_then(|t| t.get("Prev"))
                .and_then(|v| v.as_number())
                .map(|n| n as usize);

            let prev_offset = match prev_offset {
                Some(o) if o > 0 && !visited.contains(&o) => o,
                _ => break,
            };
            visited.insert(prev_offset);

            // Save current entries
            let current_entries = self.xref.as_ref()
                .map(|x| x.entries.clone())
                .unwrap_or_default();
            let current_trailer = self.xref.as_ref()
                .and_then(|x| x.trailer.clone());

            // Parse xref at Prev offset
            if self.parse_xref_at(prev_offset).is_ok() {
                // Merge: current entries override prev entries
                let mut merged = std::collections::HashMap::new();
                // Add older (prev) entries first
                if let Some(ref xref) = self.xref {
                    for e in &xref.entries {
                        merged.insert((e.object_id, e.generation), *e);
                    }
                }
                // Override with newer (current) entries
                for e in &current_entries {
                    merged.insert((e.object_id, e.generation), *e);
                }
                let entries: Vec<XRefEntry> = merged.into_values().collect();

                // Also check /XRefStm in prev trailer
                let prev_xrefstm = self.xref.as_ref()
                    .and_then(|x| x.trailer.as_ref())
                    .and_then(|t| t.get("XRefStm"))
                    .and_then(|v| v.as_number())
                    .map(|n| n as usize);

                self.xref = Some(XRef {
                    entries,
                    trailer: current_trailer.or_else(|| self.xref.as_ref().and_then(|x| x.trailer.clone())),
                });

                // If there's an XRefStm (hybrid xref), parse that too
                if let Some(stm_offset) = prev_xrefstm {
                    if !visited.contains(&stm_offset) {
                        visited.insert(stm_offset);
                        let saved = self.xref.clone();
                        if self.parse_xref_at(stm_offset).is_ok() {
                            if let Some(ref stm_xref) = self.xref {
                                let mut merged = std::collections::HashMap::new();
                                for e in &stm_xref.entries {
                                    merged.insert((e.object_id, e.generation), *e);
                                }
                                if let Some(ref saved_xref) = saved {
                                    for e in &saved_xref.entries {
                                        merged.insert((e.object_id, e.generation), *e);
                                    }
                                }
                                self.xref = Some(XRef {
                                    entries: merged.into_values().collect(),
                                    trailer: saved.and_then(|s| s.trailer),
                                });
                            }
                        } else {
                            self.xref = saved;
                        }
                    }
                }
            } else {
                // Restore current state if prev parsing fails
                self.xref = Some(XRef {
                    entries: current_entries,
                    trailer: current_trailer,
                });
                break;
            }
        }
        Ok(())
    }

    pub fn parse_xref_table(&mut self) -> Result<(), OdeError> {
        let mut entries = Vec::new();

        // Parse multiple xref subsections until we hit "trailer"
        loop {
            self.skip_whitespace();

            // Check if we've reached "trailer"
            if self.pos + 7 <= self.data.len() && &self.data[self.pos..self.pos + 7] == b"trailer" {
                break;
            }

            // Check for EOF or non-numeric (safety)
            if self.pos >= self.data.len() {
                break;
            }
            let peek = self.data[self.pos];
            if !peek.is_ascii_digit() && peek != b'-' && peek != b'+' {
                break;
            }

            let first_obj = self.parse_number()?;
            let num_entries = self.parse_number()?;

            for i in 0..(num_entries as usize) {
                let offset = match self.parse_number() {
                    Ok(n) => n as u64,
                    Err(_) => {
                        return Err(OdeError::PdfParseError(
                            format!("Unexpected end of xref entries at position {}", i),
                        ))
                    }
                };
                let gen = match self.parse_number() {
                    Ok(n) => n as u16,
                    Err(_) => {
                        return Err(OdeError::PdfParseError(
                            format!("Unexpected end of xref entries at position {}", i),
                        ))
                    }
                };
                let in_use = match self.parse_keyword() {
                    Ok(k) => k == "n",
                    Err(_) => {
                        return Err(OdeError::PdfParseError(
                            format!("Unexpected end of xref entries at position {}", i),
                        ))
                    }
                };

                entries.push(XRefEntry {
                    object_id: first_obj.saturating_add(i as i64) as u64,
                    generation: gen,
                    offset,
                    in_use,
                    objstm_num: None,
                    objstm_idx: None,
                });
            }
        }

        // Parse trailer dictionary after the xref table
        let trailer = if self.try_consume(b"trailer") {
            self.skip_whitespace();
            self.parse_dictionary().ok()
        } else {
            None
        };

        // Handle /XRefStm in hybrid-reference PDFs
        if let Some(ref t) = trailer {
            if let Some(stm_offset) = t.get("XRefStm").and_then(|v| v.as_number()).map(|n| n as usize) {
                let saved_pos = self.pos;
                self.pos = stm_offset;
                if self.data[self.pos..].first().map_or(false, |b| b.is_ascii_digit()) {
                    if let Ok(stm_entries) = self.parse_xref_stream_object_entries() {
                        entries.extend(stm_entries);
                    }
                }
                self.pos = saved_pos;
            }
        }

        self.xref = Some(XRef { entries, trailer });
        Ok(())
    }

    fn parse_trailer_only(&mut self) -> Result<(), OdeError> {
        self.skip_whitespace();
        let trailer_dict = self.parse_dictionary()?;

        let size = trailer_dict.get("Size")
            .and_then(|v| v.as_number())
            .unwrap_or(0.0) as usize;

        let entries = (0..size)
            .map(|i| XRefEntry {
                object_id: i as u64,
                generation: 0,
                offset: 0,
                in_use: true,
                objstm_num: None,
                objstm_idx: None,
            })
            .collect();

        self.xref = Some(XRef { entries, trailer: Some(trailer_dict) });
        Ok(())
    }

    fn parse_xref_stream_object(&mut self) -> Result<(), OdeError> {
        let entries = self.parse_xref_stream_object_entries()?;
        // The trailer dict was stored by parse_xref_stream_object_entries
        Ok(())
    }

    /// Parse an XRef stream object and return its entries. Also stores in self.xref.
    fn parse_xref_stream_object_entries(&mut self) -> Result<Vec<XRefEntry>, OdeError> {
        // Parse "N gen obj"
        let _obj_num = self.parse_number()?;
        let _gen_num = self.parse_number()?;
        self.skip_whitespace();
        if !self.try_consume(b"obj") {
            return Err(OdeError::PdfParseError("Expected 'obj' in XRef stream".to_string()));
        }

        self.skip_whitespace();
        let dict = self.parse_dictionary()?;

        // Read stream data
        self.skip_whitespace();
        if !self.try_consume(b"stream") {
            return Err(OdeError::PdfParseError("Expected 'stream' in XRef stream".to_string()));
        }
        // Skip \r\n or \n after "stream"
        if self.pos < self.data.len() && self.data[self.pos] == b'\r' {
            self.pos += 1;
        }
        if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
            self.pos += 1;
        }

        let length = dict.get("Length")
            .and_then(|v| v.as_number())
            .unwrap_or(0.0) as usize;

        if self.pos + length > self.data.len() {
            return Err(OdeError::PdfParseError("XRef stream length exceeds file".to_string()));
        }

        let raw_stream = &self.data[self.pos..self.pos + length];

        // Decompress if needed
        let filter = dict.get("Filter").and_then(|v| v.as_name()).unwrap_or("");
        let stream_data = if filter == "FlateDecode" {
            let mut decoder = ZlibDecoder::new(raw_stream);
            let mut buf = Vec::new();
            decoder.read_to_end(&mut buf)
                .map_err(|e| OdeError::PdfParseError(format!("Failed to decompress XRef stream: {}", e)))?;
            buf
        } else {
            raw_stream.to_vec()
        };

        // Parse /W array (field widths)
        let w = dict.get("W")
            .and_then(|v| v.as_array())
            .ok_or_else(|| OdeError::PdfParseError("Missing /W in XRef stream".to_string()))?;
        if w.len() != 3 {
            return Err(OdeError::PdfParseError("Invalid /W array length".to_string()));
        }
        let w1 = w[0].as_number().unwrap_or(0.0) as usize;
        let w2 = w[1].as_number().unwrap_or(0.0) as usize;
        let w3 = w[2].as_number().unwrap_or(0.0) as usize;
        let entry_size = w1 + w2 + w3;

        if entry_size == 0 {
            self.xref = Some(XRef { entries: Vec::new(), trailer: Some(dict) });
            return Ok(Vec::new());
        }

        // Parse /Index array or default to [0, Size]
        let index_ranges = if let Some(idx_arr) = dict.get("Index").and_then(|v| v.as_array()) {
            let mut ranges = Vec::new();
            let mut i = 0;
            while i + 1 < idx_arr.len() {
                let start = idx_arr[i].as_number().unwrap_or(0.0) as u64;
                let count = idx_arr[i + 1].as_number().unwrap_or(0.0) as u64;
                ranges.push((start, count));
                i += 2;
            }
            ranges
        } else {
            let size = dict.get("Size").and_then(|v| v.as_number()).unwrap_or(0.0) as u64;
            vec![(0, size)]
        };

        // Parse binary entries
        let mut entries = Vec::new();
        let mut stream_pos = 0;

        for (start_id, count) in &index_ranges {
            for i in 0..*count {
                if stream_pos + entry_size > stream_data.len() {
                    break;
                }

                let entry_type = if w1 > 0 {
                    read_be_uint(&stream_data[stream_pos..stream_pos + w1])
                } else {
                    1 // Default type is 1 (normal) when w1 is 0
                };
                let field2 = if w2 > 0 {
                    read_be_uint(&stream_data[stream_pos + w1..stream_pos + w1 + w2])
                } else {
                    0
                };
                let field3 = if w3 > 0 {
                    read_be_uint(&stream_data[stream_pos + w1 + w2..stream_pos + w1 + w2 + w3])
                } else {
                    0
                };

                stream_pos += entry_size;

                match entry_type {
                    0 => {
                        // Free entry
                        entries.push(XRefEntry {
                            object_id: start_id + i,
                            generation: field3 as u16,
                            offset: field2,
                            in_use: false,
                            objstm_num: None,
                            objstm_idx: None,
                        });
                    }
                    1 => {
                        // Normal (uncompressed) entry
                        entries.push(XRefEntry {
                            object_id: start_id + i,
                            generation: field3 as u16,
                            offset: field2,
                            in_use: true,
                            objstm_num: None,
                            objstm_idx: None,
                        });
                    }
                    2 => {
                        // Compressed entry in object stream
                        // field2 = object stream number, field3 = index in stream
                        entries.push(XRefEntry {
                            object_id: start_id + i,
                            generation: 0,
                            offset: 0,
                            in_use: true,
                            objstm_num: Some(field2),
                            objstm_idx: Some(field3),
                        });
                    }
                    _ => {
                        // Unknown type, skip
                    }
                }
            }
        }

        self.xref = Some(XRef { entries: entries.clone(), trailer: Some(dict) });
        Ok(entries)
    }

    fn parse_dictionary(&mut self) -> Result<Dictionary, OdeError> {
        self.skip_whitespace();
        self.consume(b"<<")?;

        let mut entries = Vec::new();

        loop {
            self.skip_whitespace();

            if self.try_consume(b">>") {
                break;
            }

            let key = self.parse_name()?;

            self.skip_whitespace();
            let value = self.parse_object()?;

            entries.push((key, value));
        }

        Ok(Dictionary { entries })
    }

    pub fn parse_object(&mut self) -> Result<PdfObject, OdeError> {
        self.skip_whitespace();

        let peek = self.peek_byte();

        if peek == Some(b'(') {
            Ok(PdfObject::String(self.parse_literal_string()?))
        } else if peek == Some(b'<') && self.peek_bytes(2).map_or(false, |b| b == [b'<', b'<']) {
            Ok(PdfObject::Dictionary(self.parse_dictionary()?))
        } else if peek == Some(b'[') {
            Ok(PdfObject::Array(self.parse_array()?))
        } else if peek == Some(b'<') {
            Ok(PdfObject::String(self.parse_hex_string()?))
        } else if peek == Some(b'/') {
            Ok(PdfObject::Name(self.parse_name()?))
        } else {
            let token = self.parse_token()?;
            match token.as_str() {
                "true" => Ok(PdfObject::Boolean(true)),
                "false" => Ok(PdfObject::Boolean(false)),
                "null" => Ok(PdfObject::Null),
                s => {
                    if let Ok(n) = s.parse::<i64>() {
                        // Check for indirect reference: N gen R
                        let saved_pos = self.pos;
                        self.skip_whitespace();
                        let gen_token = self.parse_token().unwrap_or_default();
                        if let Ok(gen) = gen_token.parse::<u16>() {
                            self.skip_whitespace();
                            if self.try_consume(b"R") {
                                return Ok(PdfObject::IndirectReference { obj_id: n as u64, gen });
                            }
                        }
                        // Not an indirect reference, restore position
                        self.pos = saved_pos;
                        Ok(PdfObject::Integer(n))
                    } else if let Ok(f) = s.parse::<f64>() {
                        Ok(PdfObject::Float(f))
                    } else {
                        Err(OdeError::PdfParseError(format!("Unknown token: {}", token)))
                    }
                }
            }
        }
    }

    fn parse_array(&mut self) -> Result<Vec<PdfObject>, OdeError> {
        self.consume(b"[")?;

        let mut items = Vec::new();

        loop {
            self.skip_whitespace();

            if self.try_consume(b"]") {
                break;
            }

            items.push(self.parse_object()?);
        }

        Ok(items)
    }

    fn parse_literal_string(&mut self) -> Result<String, OdeError> {
        self.consume(b"(")?;

        let mut result = String::new();
        let mut depth = 1;

        while depth > 0 && self.pos < self.data.len() {
            let byte = self.data[self.pos];
            self.pos += 1;

            if byte == b'(' {
                depth += 1;
            } else if byte == b')' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            } else if byte == b'\\' {
                if self.pos < self.data.len() {
                    let escaped = self.data[self.pos];
                    if escaped.is_ascii_digit() && escaped <= b'7' {
                        // Octal escape: consume up to 3 octal digits
                        let mut octal_val: u16 = (escaped - b'0') as u16;
                        self.pos += 1;
                        for _ in 0..2 {
                            if self.pos < self.data.len() {
                                let d = self.data[self.pos];
                                if d.is_ascii_digit() && d <= b'7' {
                                    octal_val = octal_val * 8 + (d - b'0') as u16;
                                    self.pos += 1;
                                } else {
                                    break;
                                }
                            }
                        }
                        result.push((octal_val & 0xFF) as u8 as char);
                    } else {
                        self.pos += 1;
                        let ch = match escaped {
                            b'n' => '\n',
                            b'r' => '\r',
                            b't' => '\t',
                            b'b' => '\x08',
                            b'f' => '\x0c',
                            b'(' => '(',
                            b')' => ')',
                            b'\\' => '\\',
                            d => d as char,
                        };
                        result.push(ch);
                    }
                }
            } else {
                result.push(byte as char);
            }
        }

        Ok(result)
    }

    fn parse_hex_string(&mut self) -> Result<String, OdeError> {
        self.consume(b"<")?;

        let mut hex_chars = Vec::new();

        while self.pos < self.data.len() {
            let byte = self.data[self.pos];

            if byte == b'>' {
                self.pos += 1;
                break;
            } else if byte.is_ascii_hexdigit() {
                hex_chars.push(byte);
            }
            self.pos += 1;
        }

        if hex_chars.len() % 2 == 1 {
            hex_chars.push(b'0');
        }

        let mut result = String::new();
        for chunk in hex_chars.chunks(2) {
            let byte_str = std::str::from_utf8(chunk).unwrap_or("00");
            if let Ok(byte_val) = u8::from_str_radix(byte_str, 16) {
                result.push(byte_val as char);
            }
        }

        Ok(result)
    }

    fn parse_name(&mut self) -> Result<String, OdeError> {
        self.consume(b"/")?;

        let mut name = String::new();

        while self.pos < self.data.len() {
            let byte = self.data[self.pos];

            // PDF delimiter characters end a name token
            if byte.is_ascii_whitespace()
                || byte == b'/'
                || byte == b'<'
                || byte == b'>'
                || byte == b'['
                || byte == b']'
                || byte == b'('
                || byte == b')'
                || byte == b'{'
                || byte == b'}'
                || byte == b'%'
            {
                break;
            }

            self.pos += 1;

            if byte == b'#' && self.pos + 1 < self.data.len() {
                let hex1 = self.data[self.pos];
                let hex2 = self.data[self.pos + 1];
                self.pos += 2;

                if hex1.is_ascii_hexdigit() && hex2.is_ascii_hexdigit() {
                    let hex_bytes = [hex1, hex2];
                    let hex_str = std::str::from_utf8(&hex_bytes).unwrap_or("00");
                    if let Ok(byte_val) = u8::from_str_radix(hex_str, 16) {
                        name.push(byte_val as char);
                    }
                }
            } else if byte.is_ascii() {
                name.push(byte as char);
            }
        }

        Ok(name)
    }

    pub fn parse_token(&mut self) -> Result<String, OdeError> {
        self.skip_whitespace();

        let start = self.pos;

        while self.pos < self.data.len()
            && !self.data[self.pos].is_ascii_whitespace()
            && self.data[self.pos] != b'<'
            && self.data[self.pos] != b'>'
            && self.data[self.pos] != b'['
            && self.data[self.pos] != b']'
            && self.data[self.pos] != b'('
            && self.data[self.pos] != b')'
            && self.data[self.pos] != b'/'
        {
            self.pos += 1;
        }

        if start >= self.pos {
            return Err(OdeError::PdfParseError("Expected token".to_string()));
        }

        Ok(std::str::from_utf8(&self.data[start..self.pos])
            .unwrap_or("")
            .to_string())
    }

    fn parse_number(&mut self) -> Result<i64, OdeError> {
        let token = self.parse_token()?;
        token
            .parse::<i64>()
            .map_err(|_| OdeError::PdfParseError(format!("Expected number, got: {}", token)))
    }

    fn parse_keyword(&mut self) -> Result<String, OdeError> {
        self.parse_token()
    }

    pub fn skip_whitespace(&mut self) {
        while self.pos < self.data.len() {
            if self.data[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            } else if self.data[self.pos] == b'%' {
                // Skip PDF comment (% to end of line)
                while self.pos < self.data.len()
                    && self.data[self.pos] != b'\n'
                    && self.data[self.pos] != b'\r'
                {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    pub fn consume(&mut self, expected: &[u8]) -> Result<(), OdeError> {
        self.skip_whitespace();

        if self.pos + expected.len() > self.data.len() {
            return Err(OdeError::PdfParseError(
                "Unexpected end of file".to_string(),
            ));
        }

        if &self.data[self.pos..self.pos + expected.len()] == expected {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(OdeError::PdfParseError(format!(
                "Expected {:?}, found different",
                String::from_utf8_lossy(expected)
            )))
        }
    }

    pub fn try_consume(&mut self, expected: &[u8]) -> bool {
        if self.pos + expected.len() <= self.data.len()
            && &self.data[self.pos..self.pos + expected.len()] == expected
        {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }

    pub fn peek_byte(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    fn peek_bytes(&self, n: usize) -> Option<&[u8]> {
        self.data.get(self.pos..self.pos + n)
    }
}

/// Read a big-endian unsigned integer from n bytes
fn read_be_uint(bytes: &[u8]) -> u64 {
    let mut result = 0u64;
    for &b in bytes {
        result = (result << 8) | (b as u64);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let data = b"%PDF-1.4\n";
        let parser = PdfParser::new(data);
        assert_eq!(parser.pos, 0);
    }

    #[test]
    fn test_skip_header() {
        let data = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF";
        let parser = &mut PdfParser::new(data);
        assert!(parser.skip_pdf_header().is_ok());
        assert!(parser.pos > 0);
    }

    #[test]
    fn test_find_xref_offset() {
        let data = b"startxref\n42\n%%EOF";
        let parser = PdfParser::new(data);
        assert_eq!(parser.find_xref_offset(), Some(42));
    }

    #[test]
    fn test_parse_hex_string() {
        let data = b"<48656C6C6F>";
        let parser = &mut PdfParser::new(data);
        let result = parser.parse_hex_string();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello");
    }

    #[test]
    fn test_parse_name() {
        let data = b"/Type";
        let parser = &mut PdfParser::new(data);
        let result = parser.parse_name();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Type");
    }

    #[test]
    fn test_parse_token() {
        let data = b"42";
        let parser = &mut PdfParser::new(data);
        let result = parser.parse_token();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_parse_simple_dictionary() {
        let data = b"<< /Type /Page >>";
        let parser = &mut PdfParser::new(data);
        let result = parser.parse_dictionary();
        assert!(result.is_ok());
        let dict = result.unwrap();
        assert!(!dict.entries.is_empty());
    }
}
