#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub transparent: bool,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            transparent: false,
            r,
            g,
            b,
        }
    }

    pub fn transparent() -> Self {
        Self {
            transparent: true,
            r: 0,
            g: 0,
            b: 0,
        }
    }

    pub fn from_rgb_normalized(r: f64, g: f64, b: f64) -> Self {
        Self {
            transparent: false,
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    pub fn to_rgb_normalized(&self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }

    pub fn distance(&self, other: &Self) -> f64 {
        if self.transparent && other.transparent {
            return 0.0;
        }
        if self.transparent || other.transparent {
            return 1.0;
        }

        let (r1, g1, b1) = self.to_rgb_normalized();
        let (r2, g2, b2) = other.to_rgb_normalized();

        let dr = r1 - r2;
        let dg = g1 - g2;
        let db = b1 - b2;

        (dr * dr + dg * dg + db * db).sqrt()
    }

    pub fn to_css_string(&self) -> String {
        if self.transparent {
            "transparent".to_string()
        } else {
            format!("rgb({}, {}, {})", self.r, self.g, self.b)
        }
    }

    pub fn from_css_string(s: &str) -> Self {
        if s == "transparent" {
            let mut c = Self::new(0, 0, 0);
            c.transparent = true;
            return c;
        }
        // Parse "rgb(r, g, b)"
        let s = s.trim_start_matches("rgb(").trim_end_matches(')');
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() == 3 {
            let r = parts[0].trim().parse::<u8>().unwrap_or(0);
            let g = parts[1].trim().parse::<u8>().unwrap_or(0);
            let b = parts[2].trim().parse::<u8>().unwrap_or(0);
            Self::new(r, g, b)
        } else {
            Self::new(0, 0, 0)
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let c = Color::new(255, 128, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert!(!c.transparent);
    }

    #[test]
    fn test_transparent_color() {
        let c = Color::transparent();
        assert!(c.transparent);
    }

    #[test]
    fn test_color_distance() {
        let c1 = Color::new(0, 0, 0);
        let c2 = Color::new(255, 255, 255);
        let dist = c1.distance(&c2);
        assert!((dist - 1.732).abs() < 0.01);
    }

    #[test]
    fn test_color_css_string() {
        let c = Color::new(255, 128, 64);
        assert_eq!(c.to_css_string(), "rgb(255, 128, 64)");
    }

    #[test]
    fn test_transparent_css_string() {
        let c = Color::transparent();
        assert_eq!(c.to_css_string(), "transparent");
    }
}
