use std::collections::HashMap;

pub struct LigatureMapper {
    ligatures: HashMap<char, &'static str>,
    emoji_conflicts: HashMap<char, char>,
}

impl LigatureMapper {
    pub fn new() -> Self {
        let mut ligatures = HashMap::new();

        // Common ligatures
        ligatures.insert('ﬁ', "fi");
        ligatures.insert('ﬂ', "fl");
        ligatures.insert('ﬀ', "ff");
        ligatures.insert('ﬃ', "ffi");
        ligatures.insert('ﬄ', "ffl");
        ligatures.insert('ﬅ', "ft");
        ligatures.insert('ﬆ', "st");
        ligatures.insert('﬇', "st");
        ligatures.insert('﬈', "st");
        ligatures.insert('﬉', "st");
        ligatures.insert('﬊', "st");
        ligatures.insert('﬋', "st");
        ligatures.insert('﬌', "st");
        ligatures.insert('﬍', "st");
        ligatures.insert('﬎', "st");
        ligatures.insert('﬏', "st");
        ligatures.insert('﬐', "st");
        ligatures.insert('﬑', "st");
        ligatures.insert('﬒', "st");
        ligatures.insert('ﬓ', "st");
        ligatures.insert('ﬔ', "st");
        ligatures.insert('ﬕ', "st");
        ligatures.insert('ﬖ', "st");
        ligatures.insert('ﬗ', "st");
        ligatures.insert('﬘', "st");
        ligatures.insert('﬙', "st");
        ligatures.insert('﬚', "st");
        ligatures.insert('נּ', "fa");
        ligatures.insert('סּ', "fa");
        ligatures.insert('﭂', "fa");
        ligatures.insert('ףּ', "fa");
        ligatures.insert('פּ', "fa");
        ligatures.insert('﭅', "fa");
        ligatures.insert('צּ', "fa");
        ligatures.insert('קּ', "fa");
        ligatures.insert('רּ', "fa");
        ligatures.insert('שּ', "fa");
        ligatures.insert('תּ', "fa");
        ligatures.insert('וֹ', "fa");
        ligatures.insert('בֿ', "fa");
        ligatures.insert('כֿ', "ff");
        ligatures.insert('פֿ', "ff");
        ligatures.insert('ﭏ', "ff");
        ligatures.insert('ﭐ', "ffi");
        ligatures.insert('ﭑ', "ff");
        ligatures.insert('ﭒ', "fl");
        ligatures.insert('ﭓ', "fl");
        ligatures.insert('ﭔ', "fl");
        ligatures.insert('ﭕ', "fl");
        ligatures.insert('ﭖ', "fl");
        ligatures.insert('ﭗ', "fl");
        ligatures.insert('ﭘ', "fl");
        ligatures.insert('ﭙ', "fl");
        ligatures.insert('ﭚ', "fl");
        ligatures.insert('ﭛ', "fl");
        ligatures.insert('ﭜ', "fl");
        ligatures.insert('ﭝ', "fl");
        ligatures.insert('ﭞ', "fl");
        ligatures.insert('ﭟ', "fl");
        ligatures.insert('ﭠ', "fl");
        ligatures.insert('ﭡ', "fl");
        ligatures.insert('ﭢ', "fl");
        ligatures.insert('ﭣ', "fl");
        ligatures.insert('ﭤ', "fl");
        ligatures.insert('ﭥ', "fl");
        ligatures.insert('ﭦ', "fl");
        ligatures.insert('ﭧ', "fl");
        ligatures.insert('ﭨ', "fl");
        ligatures.insert('ﭩ', "fl");
        ligatures.insert('ﭪ', "fl");
        ligatures.insert('ﭫ', "fl");
        ligatures.insert('ﭬ', "fl");
        ligatures.insert('ﭭ', "fl");

        // Unicode Private Use Area range for emoji conflicts (U+E000-U+E5FF)
        let mut emoji_conflicts = HashMap::new();
        emoji_conflicts.insert('\u{E000}', '\u{1F600}'); // E000 -> 😀
        emoji_conflicts.insert('\u{E001}', '\u{1F601}');
        emoji_conflicts.insert('\u{E002}', '\u{1F602}');
        emoji_conflicts.insert('\u{E003}', '\u{1F603}');
        emoji_conflicts.insert('\u{E004}', '\u{1F604}');
        emoji_conflicts.insert('\u{E005}', '\u{1F605}');
        emoji_conflicts.insert('\u{E006}', '\u{1F606}');
        emoji_conflicts.insert('\u{E007}', '\u{1F609}');
        emoji_conflicts.insert('\u{E008}', '\u{1F60A}');
        emoji_conflicts.insert('\u{E009}', '\u{1F60B}');
        emoji_conflicts.insert('\u{E00A}', '\u{1F60C}');
        emoji_conflicts.insert('\u{E00B}', '\u{1F60D}');
        emoji_conflicts.insert('\u{E00C}', '\u{1F60E}');
        emoji_conflicts.insert('\u{E00D}', '\u{1F60F}');
        emoji_conflicts.insert('\u{E00E}', '\u{1F610}');
        emoji_conflicts.insert('\u{E00F}', '\u{1F611}');
        emoji_conflicts.insert('\u{E010}', '\u{1F612}');
        emoji_conflicts.insert('\u{E011}', '\u{1F613}');
        emoji_conflicts.insert('\u{E012}', '\u{1F614}');
        emoji_conflicts.insert('\u{E013}', '\u{1F615}');
        emoji_conflicts.insert('\u{E014}', '\u{1F616}');
        emoji_conflicts.insert('\u{E015}', '\u{1F617}');
        emoji_conflicts.insert('\u{E016}', '\u{1F618}');
        emoji_conflicts.insert('\u{E017}', '\u{1F619}');
        emoji_conflicts.insert('\u{E018}', '\u{1F61A}');
        emoji_conflicts.insert('\u{E019}', '\u{1F61B}');
        emoji_conflicts.insert('\u{E01A}', '\u{1F61C}');
        emoji_conflicts.insert('\u{E01B}', '\u{1F61D}');
        emoji_conflicts.insert('\u{E01C}', '\u{1F61E}');

        Self {
            ligatures,
            emoji_conflicts,
        }
    }

    pub fn decompose(&self, c: char) -> String {
        if let Some(expansion) = self.ligatures.get(&c) {
            expansion.to_string()
        } else {
            c.to_string()
        }
    }

    pub fn decompose_string(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);
        for c in text.chars() {
            if let Some(expansion) = self.ligatures.get(&c) {
                result.push_str(expansion);
            } else {
                result.push(c);
            }
        }
        result
    }

    pub fn map_to_safe_unicode(&self, c: char) -> char {
        // Check if it's a problematic character
        if self.is_problematic_unicode(c) {
            // Map to PUA range
            let code = c as u32;
            if code >= 0xE000 && code <= 0xE5FF {
                c // Already in safe PUA range
            } else {
                // Map to PUA
                let safe_code = ((code % 0x600) + 0xE000) as u32;
                std::char::from_u32(safe_code).unwrap_or(c)
            }
        } else {
            c
        }
    }

    pub fn is_problematic_unicode(&self, c: char) -> bool {
        let code = c as u32;

        // Control characters (C0 controls) except whitespace
        if code < 0x20 {
            return code != 0x09 && code != 0x0A && code != 0x0D; // Allow tab, newline, CR
        }

        // C1 controls
        if code >= 0x7F && code <= 0x9F {
            return true;
        }

        // Bidirectional formatting characters
        if code >= 0x200E && code <= 0x200F {
            return true;
        }
        if code >= 0x202A && code <= 0x202E {
            return true;
        }

        // Zero-width characters
        if code == 0x200B || code == 0x200C || code == 0x200D {
            return true;
        }
        if code == 0xFEFF {
            return true;
        }

        // In the emoji conflict range
        if code >= 0xE000 && code <= 0xE5FF {
            return true;
        }

        false
    }

    pub fn escape_problematic(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);
        for c in text.chars() {
            if self.is_problematic_unicode(c) {
                let safe = self.map_to_safe_unicode(c);
                // Use numeric character reference for safety
                result.push_str(&format!("&#x{:X};", safe as u32));
            } else {
                decompose_char_to_string(&mut result, c, &self.ligatures);
            }
        }
        result
    }
}

fn decompose_char_to_string(result: &mut String, c: char, ligatures: &HashMap<char, &'static str>) {
    if let Some(expansion) = ligatures.get(&c) {
        result.push_str(expansion);
    } else {
        result.push(c);
    }
}

impl Default for LigatureMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ligature_decomposition() {
        let mapper = LigatureMapper::new();
        assert_eq!(mapper.decompose('ﬁ'), "fi");
        assert_eq!(mapper.decompose('ﬂ'), "fl");
        assert_eq!(mapper.decompose('ﬀ'), "ff");
        assert_eq!(mapper.decompose('ﬃ'), "ffi");
        assert_eq!(mapper.decompose('A'), "A");
    }

    #[test]
    fn test_string_decomposition() {
        let mapper = LigatureMapper::new();
        assert_eq!(mapper.decompose_string("ﬁrst"), "first");
        assert_eq!(mapper.decompose_string("ﬂow"), "flow");
        assert_eq!(mapper.decompose_string("Hello"), "Hello");
    }

    #[test]
    fn test_problematic_unicode_detection() {
        let mapper = LigatureMapper::new();
        assert!(mapper.is_problematic_unicode('\x00'));
        assert!(mapper.is_problematic_unicode('\x1F'));
        assert!(!mapper.is_problematic_unicode('\x09')); // Tab is OK
        assert!(!mapper.is_problematic_unicode('\x0A')); // Newline is OK
        assert!(!mapper.is_problematic_unicode('A'));
    }

    #[test]
    fn test_escape_problematic() {
        let mapper = LigatureMapper::new();
        let result = mapper.escape_problematic("\x00ﬁ");
        assert!(result.contains("&#x"));
    }
}
