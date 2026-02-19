use crate::types::color::Color;
use crate::util::math::TransformMatrix;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CssClass {
    pub name: String,
    pub properties: String,
}

#[derive(Debug, Clone)]
pub struct StyleManager {
    font_size_classes: HashMap<u32, String>,
    color_classes: HashMap<u32, String>,
    transform_classes: HashMap<String, String>,
    class_counter: u32,
}

impl StyleManager {
    pub fn new() -> Self {
        Self {
            font_size_classes: HashMap::new(),
            color_classes: HashMap::new(),
            transform_classes: HashMap::new(),
            class_counter: 0,
        }
    }

    pub fn get_font_size_class(&mut self, font_size: f64) -> String {
        let key = (font_size * 100.0) as u32;

        if let Some(class) = self.font_size_classes.get(&key) {
            return class.clone();
        }

        let class_name = format!("fs{}", self.class_counter);
        self.class_counter += 1;

        let _properties = format!("font-size:{:.2}px;", font_size);
        self.font_size_classes.insert(key, class_name.clone());

        class_name
    }

    pub fn get_color_class(&mut self, color: &Color) -> String {
        let key = color.to_hash();

        if let Some(class) = self.color_classes.get(&key) {
            return class.clone();
        }

        let class_name = format!("c{}", self.class_counter);
        self.class_counter += 1;

        let _properties = format!("color:{};", color.to_css_string());
        self.color_classes.insert(key, class_name.clone());

        class_name
    }

    pub fn get_transform_class(&mut self, tm: &TransformMatrix) -> String {
        let key = tm.to_key();

        if let Some(class) = self.transform_classes.get(&key) {
            return class.clone();
        }

        let class_name = format!("t{}", self.class_counter);
        self.class_counter += 1;

        self.transform_classes.insert(key, class_name.clone());

        class_name
    }

    pub fn generate_css(&self) -> String {
        let mut css = String::new();

        // Font size classes
        for (key, class_name) in &self.font_size_classes {
            if let Some(class) = self.get_font_size_css(*key, class_name) {
                css.push_str(&class);
                css.push('\n');
            }
        }

        // Color classes
        for (key, class_name) in &self.color_classes {
            if let Some(class) = self.get_color_css(*key, class_name) {
                css.push_str(&class);
                css.push('\n');
            }
        }

        // Transform classes
        for (key, class_name) in &self.transform_classes {
            if let Some(class) = self.get_transform_css(key, class_name) {
                css.push_str(&class);
                css.push('\n');
            }
        }

        css
    }

    fn get_font_size_css(&self, key: u32, class_name: &str) -> Option<String> {
        let font_size = key as f64 / 100.0;
        Some(format!(
            ".{} {{\n  font-size:{:.2}px;\n}}",
            class_name, font_size
        ))
    }

    fn get_color_css(&self, key: u32, class_name: &str) -> Option<String> {
        let color = Color::from_hash(key)?;
        Some(format!(
            ".{} {{\n  color:{};\n}}",
            class_name,
            color.to_css_string()
        ))
    }

    fn get_transform_css(&self, _key: &str, class_name: &str) -> Option<String> {
        // This would need the actual matrix, simplified for now
        Some(format!(
            ".{} {{\n  transform:matrix(1,0,0,1,0,0);\n}}",
            class_name
        ))
    }

    pub fn clear(&mut self) {
        self.font_size_classes.clear();
        self.color_classes.clear();
        self.transform_classes.clear();
        self.class_counter = 0;
    }

    pub fn class_count(&self) -> usize {
        self.font_size_classes.len() + self.color_classes.len() + self.transform_classes.len()
    }
}

impl Default for StyleManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Color {
    pub fn to_hash(&self) -> u32 {
        let r = self.r as u32;
        let g = self.g as u32;
        let b = self.b as u32;
        let trans = if self.transparent { 1u32 } else { 0u32 };

        (r << 16) | (g << 8) | b | trans
    }

    pub fn from_hash(hash: u32) -> Option<Self> {
        let r = ((hash >> 16) & 0xFF) as u8;
        let g = ((hash >> 8) & 0xFF) as u8;
        let b = (hash & 0xFF) as u8;
        let trans = hash & 1 == 1;

        Some(Color {
            r,
            g,
            b,
            transparent: trans,
        })
    }
}

impl TransformMatrix {
    pub fn to_key(&self) -> String {
        format!(
            "{:.3}|{:.3}|{:.3}|{:.3}|{:.3}|{:.3}",
            self.a, self.b, self.c, self.d, self.e, self.f
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_manager_creation() {
        let manager = StyleManager::new();
        assert_eq!(manager.class_count(), 0);
    }

    #[test]
    fn test_font_size_class_generation() {
        let mut manager = StyleManager::new();

        let class1 = manager.get_font_size_class(12.0);
        let class2 = manager.get_font_size_class(12.0);
        let class3 = manager.get_font_size_class(14.0);

        assert_eq!(class1, class2); // Same font size, same class
        assert_ne!(class1, class3); // Different font size, different class
        assert!(manager.class_count() >= 1);
    }

    #[test]
    fn test_color_class_generation() {
        let mut manager = StyleManager::new();

        let color1 = Color::new(255, 0, 0);
        let color2 = Color::new(255, 0, 0);
        let color3 = Color::new(0, 255, 0);

        let class1 = manager.get_color_class(&color1);
        let class2 = manager.get_color_class(&color2);
        let class3 = manager.get_color_class(&color3);

        assert_eq!(class1, class2); // Same color, same class
        assert_ne!(class1, class3); // Different color, different class
    }

    #[test]
    fn test_color_hash() {
        let color1 = Color::new(255, 128, 64);
        let hash1 = color1.to_hash();
        let color2 = Color::from_hash(hash1).unwrap();

        assert_eq!(color1.r, color2.r);
        assert_eq!(color1.g, color2.g);
        assert_eq!(color1.b, color2.b);
        assert_eq!(color1.transparent, color2.transparent);
    }

    #[test]
    fn test_transform_class_generation() {
        let mut manager = StyleManager::new();

        let tm1 = TransformMatrix::identity();
        let tm2 = TransformMatrix::identity();
        let mut tm3 = TransformMatrix::identity();
        tm3.a = 2.0;

        let class1 = manager.get_transform_class(&tm1);
        let class2 = manager.get_transform_class(&tm2);
        let class3 = manager.get_transform_class(&tm3);

        assert_eq!(class1, class2); // Same transform, same class
        assert_ne!(class1, class3); // Different transform, different class
    }

    #[test]
    fn test_css_generation() {
        let mut manager = StyleManager::new();

        manager.get_font_size_class(12.0);
        manager.get_color_class(&Color::new(255, 0, 0));

        let css = manager.generate_css();
        assert!(!css.is_empty());
        assert!(css.contains("font-size"));
        assert!(css.contains("color"));
    }

    #[test]
    fn test_clear() {
        let mut manager = StyleManager::new();

        manager.get_font_size_class(12.0);
        manager.clear();

        assert_eq!(manager.class_count(), 0);
    }
}
