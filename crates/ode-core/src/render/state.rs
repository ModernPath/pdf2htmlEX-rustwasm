use crate::types::color::Color;
use crate::util::math::TransformMatrix;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct FontInfo {
    pub id: u64,
    pub use_tounicode: bool,
    pub em_size: f64,
    pub space_width: f64,
    pub ascent: f64,
    pub descent: f64,
    pub is_type3: bool,
    pub font_size_scale: f64,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            id: 0,
            use_tounicode: true,
            em_size: 1000.0,
            space_width: 250.0,
            ascent: 0.8,
            descent: -0.2,
            is_type3: false,
            font_size_scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsState {
    pub font_info: Option<FontInfo>,
    pub font_size: f64,
    pub fill_color: Color,
    pub stroke_color: Color,
    pub letter_space: f64,
    pub word_space: f64,
    pub transform_matrix: TransformMatrix,
    pub clipping_enabled: bool,
}

impl GraphicsState {
    pub fn new() -> Self {
        Self {
            font_info: None,
            font_size: 12.0,
            fill_color: Color::new(0, 0, 0),
            stroke_color: Color::new(0, 0, 0),
            letter_space: 0.0,
            word_space: 0.0,
            transform_matrix: TransformMatrix::identity(),
            clipping_enabled: false,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn clone_without_line_position(&self) -> Self {
        let mut state = self.clone();
        state.transform_matrix = TransformMatrix::identity();
        state
    }

    pub fn single_space_offset(&self) -> f64 {
        let mut offset = self.word_space + self.letter_space;
        if let Some(ref font_info) = self.font_info {
            if font_info.em_size > 0.0 {
                offset += font_info.space_width * self.font_size;
            }
        }
        offset
    }

    pub fn em_size(&self) -> f64 {
        if let Some(ref font_info) = self.font_info {
            self.font_size * (font_info.ascent - font_info.descent)
        } else {
            self.font_size
        }
    }
}

impl Default for GraphicsState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for GraphicsState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GraphicsState {{ font_size: {:.2}, fill: {}, stroke: {}, transform: [{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}] }}",
            self.font_size,
            self.fill_color.to_css_string(),
            self.stroke_color.to_css_string(),
            self.transform_matrix.a, self.transform_matrix.b,
            self.transform_matrix.c, self.transform_matrix.d,
            self.transform_matrix.e, self.transform_matrix.f
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClipState {
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
}

impl ClipState {
    pub fn new() -> Self {
        Self {
            xmin: f64::NEG_INFINITY,
            xmax: f64::INFINITY,
            ymin: f64::NEG_INFINITY,
            ymax: f64::INFINITY,
        }
    }

    pub fn intersect(&mut self, other: &ClipState) {
        self.xmin = self.xmin.max(other.xmin);
        self.xmax = self.xmax.min(other.xmax);
        self.ymin = self.ymin.max(other.ymin);
        self.ymax = self.ymax.min(other.ymax);
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.xmin && x <= self.xmax && y >= self.ymin && y <= self.ymax
    }
}

impl Default for ClipState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_state_creation() {
        let state = GraphicsState::new();
        assert_eq!(state.font_size, 12.0);
        assert!(state.fill_color == Color::new(0, 0, 0));
    }

    #[test]
    fn test_font_info_default() {
        let info = FontInfo::default();
        assert_eq!(info.em_size, 1000.0);
        assert_eq!(info.font_size_scale, 1.0);
    }

    #[test]
    fn test_clip_state_intersect() {
        let mut clip1 = ClipState::new();
        clip1.xmin = 0.0;
        clip1.xmax = 100.0;
        clip1.ymin = 0.0;
        clip1.ymax = 100.0;

        let mut clip2 = ClipState::new();
        clip2.xmin = 50.0;
        clip2.xmax = 150.0;
        clip2.ymin = 50.0;
        clip2.ymax = 150.0;

        clip1.intersect(&clip2);
        assert!((clip1.xmin - 50.0).abs() < 0.001);
        assert!((clip1.xmax - 100.0).abs() < 0.001);
    }
}
