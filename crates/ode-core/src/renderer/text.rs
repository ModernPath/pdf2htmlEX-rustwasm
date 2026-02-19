use crate::error::OdeError;
use crate::render::state::GraphicsState;
use crate::types::color::Color;

#[derive(Debug, Clone)]
pub struct TextChar {
    pub unicode: char,
    pub code: u32,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub color: Color,
    pub width: f64,
}

#[derive(Debug, Clone)]
pub struct TextPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct TextSegment {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub color: Color,
    pub font_id: u64,
}

pub struct TextExtractor {
    segments: Vec<TextSegment>,
    current_segment: Option<TextSegment>,
    current_state: GraphicsState,
    last_position: Option<TextPosition>,
}

impl TextExtractor {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
            current_segment: None,
            current_state: GraphicsState::new(),
            last_position: None,
        }
    }

    pub fn update_state(&mut self, state: &GraphicsState) {
        if self.state_changed(state) {
            self.finalize_segment();
        }
        self.current_state = state.clone();
    }

    pub fn add_text(&mut self, text: &str, x: f64, y: f64) -> Result<(), OdeError> {
        if text.is_empty() {
            return Ok(());
        }

        let font_size = self.current_state.font_size;
        let color = self.current_state.fill_color.clone();
        let font_id = self
            .current_state
            .font_info
            .as_ref()
            .map(|f| f.id)
            .unwrap_or(0);

        let position_diff = if let Some(ref last) = self.last_position {
            let dx = (x - last.x).abs();
            let dy = (y - last.y).abs();
            (dx, dy)
        } else {
            (x, y)
        };

        let is_same_line = position_diff.1 < 0.5;
        let is_continuation = position_diff.0 < (font_size * 2.0) && is_same_line;

        if self.current_segment.is_some() && is_continuation {
            if let Some(ref mut seg) = self.current_segment {
                seg.text.push_str(text);
            }
        } else {
            self.finalize_segment();

            self.current_segment = Some(TextSegment {
                text: text.to_string(),
                x,
                y,
                font_size,
                color,
                font_id,
            });
        }

        self.last_position = Some(TextPosition { x, y });
        Ok(())
    }

    pub fn finalize_segment(&mut self) -> Option<TextSegment> {
        if self.current_segment.is_none() {
            return None;
        }

        let segment = self.current_segment.take()?;
        if !segment.text.trim().is_empty() {
            self.segments.push(segment.clone());
            Some(segment)
        } else {
            None
        }
    }

    pub fn get_segments(&self) -> &[TextSegment] {
        &self.segments
    }

    pub fn get_spans(&self) -> Vec<crate::renderer::TextSpan> {
        self.segments
            .iter()
            .map(|seg| crate::renderer::TextSpan {
                text: seg.text.clone(),
                x: seg.x,
                y: seg.y,
                font_size: seg.font_size,
                font_id: Some(seg.font_id),
                color: seg.color.to_css_string(),
            })
            .collect()
    }

    /// Merge pre-rendered text spans (e.g. from Form XObjects) into this extractor.
    pub fn merge_spans(&mut self, spans: &[crate::renderer::TextSpan]) {
        self.finalize_segment();
        for span in spans {
            self.segments.push(TextSegment {
                text: span.text.clone(),
                x: span.x,
                y: span.y,
                font_size: span.font_size,
                color: Color::from_css_string(&span.color),
                font_id: span.font_id.unwrap_or(0),
            });
        }
    }

    fn state_changed(&self, new_state: &GraphicsState) -> bool {
        if self.current_segment.is_none() {
            return false;
        }

        let _seg = self.current_segment.as_ref().unwrap();

        let font_changed = self.current_state.font_info != new_state.font_info;
        let color_changed = self.current_state.fill_color != new_state.fill_color;
        let size_changed = (self.current_state.font_size - new_state.font_size).abs() > 0.01;

        font_changed || color_changed || size_changed
    }
}

impl Default for TextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_extractor_creation() {
        let extractor = TextExtractor::new();
        assert!(extractor.segments.is_empty());
        assert!(extractor.current_segment.is_none());
    }

    #[test]
    fn test_add_text() -> Result<(), OdeError> {
        let mut extractor = TextExtractor::new();
        extractor.add_text("Hello", 10.0, 20.0)?;
        extractor.finalize_segment();

        assert_eq!(extractor.segments.len(), 1);
        assert_eq!(extractor.segments[0].text, "Hello");
        assert_eq!(extractor.segments[0].x, 10.0);
        assert_eq!(extractor.segments[0].y, 20.0);
        Ok(())
    }

    #[test]
    fn test_text_segmentation() -> Result<(), OdeError> {
        let mut extractor = TextExtractor::new();

        extractor.add_text("Hello", 10.0, 20.0)?;
        extractor.add_text("World", 18.0, 20.0)?;
        extractor.finalize_segment();

        assert_eq!(extractor.segments.len(), 1);
        assert!(extractor.segments[0].text.contains("Hello"));
        assert!(extractor.segments[0].text.contains("World"));
        Ok(())
    }

    #[test]
    fn test_different_lines() -> Result<(), OdeError> {
        let mut extractor = TextExtractor::new();

        extractor.add_text("Hello", 10.0, 20.0)?;
        extractor.add_text("World", 10.0, 40.0)?;
        extractor.finalize_segment();

        assert_eq!(extractor.segments.len(), 2);
        Ok(())
    }

    #[test]
    fn test_state_change() -> Result<(), OdeError> {
        let mut extractor = TextExtractor::new();
        let mut state = GraphicsState::new();

        extractor.update_state(&state);
        extractor.add_text("Hello", 10.0, 20.0)?;

        state.font_size = 14.0;
        extractor.update_state(&state);
        extractor.add_text("World", 10.0, 20.0)?;
        extractor.finalize_segment();

        assert_eq!(extractor.segments.len(), 2);
        Ok(())
    }
}
