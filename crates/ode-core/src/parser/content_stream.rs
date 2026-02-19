use crate::error::OdeError;
use crate::renderer::ContentOp;

#[derive(Debug, Clone)]
pub struct ParsedOp {
    pub operator: ContentOp,
    pub operands: Vec<f64>,
    pub text: Option<String>,
    pub text_raw: Option<Vec<u8>>,
    pub font_name: Option<String>,
}

pub struct ContentStreamParser {
    data: Vec<u8>,
    position: usize,
    operands: Vec<f64>,
    text: Option<String>,
    text_raw: Option<Vec<u8>>,
    current_font_name: Option<String>,
}

impl ContentStreamParser {
    pub fn new(data: &[u8], compression: Option<&str>) -> Result<Self, OdeError> {
        let decompressed = if let Some(comp_type) = compression {
            if comp_type == "FlateDecode" || comp_type.contains("Flate") {
                Self::decompress_flate(data)?
            } else {
                data.to_vec()
            }
        } else {
            data.to_vec()
        };

        Ok(Self {
            data: decompressed,
            position: 0,
            operands: Vec::new(),
            text: None,
            text_raw: None,
            current_font_name: None,
        })
    }

    fn decompress_flate(data: &[u8]) -> Result<Vec<u8>, OdeError> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        let mut decoder = ZlibDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| OdeError::PdfParseError(format!("Flate decompression failed: {}", e)))?;
        Ok(decompressed)
    }

    pub fn parse(&mut self) -> Result<Vec<ParsedOp>, OdeError> {
        let mut ops = Vec::new();

        while self.position < self.data.len() {
            self.skip_whitespace();

            if self.position >= self.data.len() {
                break;
            }

            if self.peek_char() == Some(b'%') {
                self.skip_comment();
                continue;
            }

            let c = self.peek_char().unwrap();

            if c.is_ascii_digit() || c == b'-' || c == b'+' || c == b'.' {
                let operand = self.parse_operand()?;
                self.operands.push(operand);
            } else if c == b'(' {
                let (text, raw) = self.parse_literal_string_with_raw()?;
                self.text = Some(text);
                self.text_raw = Some(raw);
            } else if c == b'<' {
                if self.peek_ahead(1) == Some(b'<') {
                    self.consume(b"<<");
                    self.skip_to_matching(b'>', 2)?;
                } else {
                    let (text, raw) = self.parse_hex_string_with_raw()?;
                    self.text = Some(text);
                    self.text_raw = Some(raw);
                }
            } else if c == b'/' {
                // PDF name token (e.g. /F1, /Type) — capture it for font tracking
                self.position += 1;
                let name_start = self.position;
                while self.position < self.data.len() {
                    let nc = self.data[self.position];
                    if nc.is_ascii_whitespace() || nc == b'/' || nc == b'<' || nc == b'(' || nc == b'[' || nc == b']' || nc == b'>' {
                        break;
                    }
                    self.position += 1;
                }
                let name = String::from_utf8_lossy(&self.data[name_start..self.position]).to_string();
                self.current_font_name = Some(name);
            } else {
                let op_name = self.parse_operator()?;
                if op_name.is_empty() {
                    // Unknown byte — skip to avoid infinite loop
                    self.position += 1;
                } else if let Some(operator) = ContentOp::from_name(&op_name) {
                    let font_name = if operator == ContentOp::Tf || operator == ContentOp::Do {
                        self.current_font_name.take()
                    } else {
                        None
                    };
                    ops.push(ParsedOp {
                        operator,
                        operands: std::mem::take(&mut self.operands),
                        text: self.text.take(),
                        text_raw: self.text_raw.take(),
                        font_name,
                    });
                }
            }
        }

        Ok(ops)
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.data.len() {
            let c = self.data[self.position];
            if c.is_ascii_whitespace() {
                self.position += 1;
            } else if c == b'\x00' {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while self.position < self.data.len() {
            if self.data[self.position] == b'\n' || self.data[self.position] == b'\r' {
                self.position += 1;
                break;
            }
            self.position += 1;
        }
    }

    fn peek_char(&self) -> Option<u8> {
        if self.position < self.data.len() {
            Some(self.data[self.position])
        } else {
            None
        }
    }

    fn peek_ahead(&self, offset: usize) -> Option<u8> {
        let pos = self.position + offset;
        if pos < self.data.len() {
            Some(self.data[pos])
        } else {
            None
        }
    }

    fn consume(&mut self, bytes: &[u8]) {
        for byte in bytes {
            if self.position < self.data.len() && self.data[self.position] == *byte {
                self.position += 1;
            }
        }
    }

    fn parse_operand(&mut self) -> Result<f64, OdeError> {
        let start = self.position;
        let mut has_decimal = false;
        let mut has_exponent = false;

        while self.position < self.data.len() {
            let c = self.data[self.position];
            if c.is_ascii_digit() || c == b'-' || c == b'+' {
                self.position += 1;
            } else if c == b'.' && !has_decimal {
                has_decimal = true;
                self.position += 1;
            } else if (c == b'e' || c == b'E') && !has_exponent {
                has_exponent = true;
                self.position += 1;
                if self.position < self.data.len() && self.data[self.position] == b'-' {
                    self.position += 1;
                }
            } else {
                break;
            }
        }

        let s = String::from_utf8_lossy(&self.data[start..self.position]);
        s.parse()
            .map_err(|_| OdeError::PdfParseError(format!("Invalid operand: {}", s)))
    }

    fn parse_literal_string_with_raw(&mut self) -> Result<(String, Vec<u8>), OdeError> {
        let (text, raw) = self.parse_literal_string_inner()?;
        Ok((text, raw))
    }

    fn parse_hex_string_with_raw(&mut self) -> Result<(String, Vec<u8>), OdeError> {
        let (text, raw) = self.parse_hex_string_inner()?;
        Ok((text, raw))
    }

    fn parse_literal_string_inner(&mut self) -> Result<(String, Vec<u8>), OdeError> {
        self.position += 1;
        let mut result = Vec::new();
        let mut depth = 1;

        while self.position < self.data.len() && depth > 0 {
            let c = self.data[self.position];
            self.position += 1;

            match c {
                b'(' => {
                    result.push(c);
                    depth += 1;
                }
                b')' => {
                    depth -= 1;
                    if depth > 0 {
                        result.push(c);
                    }
                }
                b'\\' => {
                    if self.position < self.data.len() {
                        let next = self.data[self.position];
                        self.position += 1;
                        result.extend(Self::escape_char(next));
                    }
                }
                _ => {
                    result.push(c);
                }
            }
        }

        // Return both the lossy string and raw bytes
        let text = String::from_utf8_lossy(&result).into_owned();
        Ok((text, result))
    }

    fn parse_hex_string_inner(&mut self) -> Result<(String, Vec<u8>), OdeError> {
        self.position += 1; // skip '<'
        let mut hex_chars = Vec::new();

        while self.position < self.data.len() && self.data[self.position] != b'>' {
            let c = self.data[self.position];
            if c.is_ascii_hexdigit() {
                hex_chars.push(c);
            }
            self.position += 1;
        }

        // Skip closing '>'
        if self.position < self.data.len() && self.data[self.position] == b'>' {
            self.position += 1;
        }

        let mut bytes = Vec::new();
        let mut i = 0;
        while i + 1 < hex_chars.len() {
            let high = hex_chars[i] as char;
            let low = hex_chars[i + 1] as char;
            let byte = (Self::hex_to_nibble(high)? << 4) | Self::hex_to_nibble(low)?;
            bytes.push(byte);
            i += 2;
        }

        // Return both the lossy string and raw bytes
        let text = String::from_utf8_lossy(&bytes).into_owned();
        Ok((text, bytes))
    }

    fn escape_char(c: u8) -> Vec<u8> {
        match c {
            b'n' => vec![b'\n'],
            b'r' => vec![b'\r'],
            b't' => vec![b'\t'],
            b'b' => vec![b'\x08'],
            b'f' => vec![b'\x0c'],
            b'(' => vec![b'('],
            b')' => vec![b')'],
            b'\\' => vec![b'\\'],
            d if d.is_ascii_digit() => {
                let n = d - b'0';
                vec![n]
            }
            _ => vec![c],
        }
    }

    fn hex_to_nibble(c: char) -> Result<u8, OdeError> {
        match c.to_ascii_uppercase() {
            '0'..='9' => Ok(c as u8 - b'0'),
            'A'..='F' => Ok(c as u8 - b'A' + 10),
            _ => Err(OdeError::PdfParseError(format!("Invalid hex: {}", c))),
        }
    }

    fn parse_operator(&mut self) -> Result<String, OdeError> {
        let start = self.position;

        while self.position < self.data.len() {
            let c = self.data[self.position];
            if c.is_ascii_whitespace() || c == b'<' || c == b'/' || c == b'(' || c == b'%' {
                break;
            }
            self.position += 1;
        }

        let op = String::from_utf8_lossy(&self.data[start..self.position]).to_string();
        Ok(op)
    }

    fn skip_to_matching(&mut self, _end: u8, depth: usize) -> Result<(), OdeError> {
        let mut depth_count = depth;
        let mut in_name = false;
        let mut in_string = false;
        let mut in_array = false;

        while self.position < self.data.len() && depth_count > 0 {
            let c = self.data[self.position];

            if !in_string && !in_array && c == b'/' {
                in_name = true;
            } else if in_name && c.is_ascii_whitespace() {
                in_name = false;
            } else if !in_name && !in_string && !in_array && c == b'<' {
                if self.peek_ahead(1) == Some(b'<') {
                    self.position += 1;
                    depth_count += 2;
                } else {
                    in_string = true;
                }
            } else if in_string && c == b'>' {
                in_string = false;
            } else if !in_name && !in_string && !in_array && c == b'>' {
                depth_count -= 2;
                if depth_count == 0 {
                    break;
                }
            } else if !in_name && !in_string && c == b'[' {
                in_array = true;
            } else if in_array && c == b']' {
                in_array = false;
            } else if in_string && c == b'\\' {
                self.position += 1;
            }

            self.position += 1;
        }

        if depth_count != 0 {
            return Err(OdeError::PdfParseError(
                "Unmatched dictionary delimiters".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_text() {
        let stream = b"BT\n/F1 12 Tf\n100 700 Td\n(Hello) Tj\nET";
        let mut parser = ContentStreamParser::new(stream, None).unwrap();
        let ops = parser.parse().unwrap();
        assert!(!ops.is_empty());
    }

    #[test]
    fn test_parse_empty_stream() {
        let stream = b"";
        let mut parser = ContentStreamParser::new(stream, None).unwrap();
        let ops = parser.parse().unwrap();
        assert!(ops.is_empty());
    }

    #[test]
    fn test_parse_whitespace() {
        let stream = b"BT  /F1  12  Tf 100  700  Td (Hello)  Tj  ET";
        let mut parser = ContentStreamParser::new(stream, None).unwrap();
        assert!(parser.parse().is_ok());
    }

    #[test]
    fn test_parse_literal_string() {
        let stream = b"(Hello World)";
        let mut parser = ContentStreamParser::new(stream, None).unwrap();
        let ops = parser.parse().unwrap();
        assert_eq!(ops.len(), 0);
    }

    #[test]
    fn test_parse_operands() {
        let stream = b"100 200 300 Td";
        let mut parser = ContentStreamParser::new(stream, None).unwrap();
        let ops = parser.parse().unwrap();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].operands, vec![100.0, 200.0, 300.0]);
    }
}
