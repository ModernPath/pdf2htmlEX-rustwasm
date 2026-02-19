use std::io::Write;

pub fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

pub fn escape_json(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c <= '\x1f' => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => result.push(c),
        }
    }
    result
}

pub fn escape_html_attribute(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            '`' => result.push_str("&#x60;"),
            _ => result.push(c),
        }
    }
    result
}

pub fn write_escaped_html<W: Write>(writer: &mut W, text: &str) -> std::io::Result<()> {
    for c in text.chars() {
        match c {
            '&' => write!(writer, "&amp;")?,
            '<' => write!(writer, "&lt;")?,
            '>' => write!(writer, "&gt;")?,
            '"' => write!(writer, "&quot;")?,
            '\'' => write!(writer, "&#39;")?,
            _ => write!(writer, "{}", c)?,
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<div>"), "&lt;div&gt;");
        assert_eq!(escape_html("&"), "&amp;");
        assert_eq!(escape_html("\""), "&quot;");
        assert_eq!(
            escape_html("<script>alert('x')</script>"),
            "&lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("Hello\nWorld"), "Hello\\nWorld");
        assert_eq!(escape_json("Quote\""), "Quote\\\"");
        assert_eq!(escape_json("Back\\slash"), "Back\\\\slash");
    }

    #[test]
    fn test_escape_html_attribute() {
        assert_eq!(
            escape_html_attribute("data\"value'"),
            "data&quot;value&#x27;"
        );
        assert_eq!(escape_html_attribute("<script>`"), "&lt;script&gt;&#x60;");
    }

    #[test]
    fn test_html_escaping_roundtrip() {
        let original = "<div class=\"test\">Hello & welcome</div>";
        let escaped = escape_html(original);
        assert_ne!(escaped, original);
    }
}
