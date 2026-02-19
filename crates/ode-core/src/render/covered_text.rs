#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CornerVisibility {
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_left: bool,
    pub bottom_right: bool,
}

impl CornerVisibility {
    pub fn all_visible() -> Self {
        Self {
            top_left: true,
            top_right: true,
            bottom_left: true,
            bottom_right: true,
        }
    }

    pub fn all_covered() -> Self {
        Self {
            top_left: false,
            top_right: false,
            bottom_left: false,
            bottom_right: false,
        }
    }

    pub fn is_any_visible(&self) -> bool {
        self.top_left || self.top_right || self.bottom_left || self.bottom_right
    }

    pub fn is_fully_visible(&self) -> bool {
        self.top_left && self.top_right && self.bottom_left && self.bottom_right
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CharBox {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl CharBox {
    pub fn new(x_min: f64, y_min: f64, x_max: f64, y_max: f64) -> Self {
        Self {
            x_min: x_min.min(x_max),
            y_min: y_min.min(y_max),
            x_max: x_min.max(x_max),
            y_max: y_min.max(y_max),
        }
    }

    pub fn center(&self) -> (f64, f64) {
        (
            (self.x_min + self.x_max) / 2.0,
            (self.y_min + self.y_max) / 2.0,
        )
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x_min && x <= self.x_max && y >= self.y_min && y <= self.y_max
    }

    pub fn intersects(&self, other: &CharBox) -> bool {
        self.x_min <= other.x_max
            && self.x_max >= other.x_min
            && self.y_min <= other.y_max
            && self.y_max >= other.y_min
    }
}

pub struct CoveredTextDetector {
    char_boxes: Vec<CharBox>,
    char_visibility: Vec<CornerVisibility>,
    drawing_ops: Vec<DrawingOp>,
}

#[derive(Debug, Clone)]
enum DrawingOp {
    Char(CharBox),
    NonChar(CharBox),
}

impl CoveredTextDetector {
    pub fn new() -> Self {
        Self {
            char_boxes: Vec::new(),
            char_visibility: Vec::new(),
            drawing_ops: Vec::new(),
        }
    }

    pub fn add_character(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let box_ = CharBox::new(x, y, x + width, y + height);
        self.char_boxes.push(box_);
        self.char_visibility.push(CornerVisibility::all_visible());
        self.drawing_ops.push(DrawingOp::Char(box_));
    }

    pub fn add_non_character(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let box_ = CharBox::new(x, y, x + width, y + height);
        self.drawing_ops.push(DrawingOp::NonChar(box_));

        // Update visibility for all previous characters
        self.update_visibility_after_non_char(&box_);
    }

    fn update_visibility_after_non_char(&mut self, non_char_box: &CharBox) {
        for (char_box, visibility) in self
            .char_boxes
            .iter_mut()
            .zip(self.char_visibility.iter_mut())
        {
            if char_box.intersects(non_char_box) {
                // Check each corner
                visibility.top_left &= !non_char_box.contains(char_box.x_min, char_box.y_max);
                visibility.top_right &= !non_char_box.contains(char_box.x_max, char_box.y_max);
                visibility.bottom_left &= !non_char_box.contains(char_box.x_min, char_box.y_min);
                visibility.bottom_right &= !non_char_box.contains(char_box.x_max, char_box.y_min);
            }
        }
    }

    pub fn get_visibility(&self, char_index: usize) -> Option<CornerVisibility> {
        self.char_visibility.get(char_index).copied()
    }

    pub fn is_char_fully_covered(&self, char_index: usize) -> bool {
        match self.char_visibility.get(char_index) {
            Some(v) => !v.is_any_visible(),
            None => false,
        }
    }

    pub fn is_char_partially_visible(&self, char_index: usize) -> bool {
        match self.char_visibility.get(char_index) {
            Some(v) => v.is_any_visible() && !v.is_fully_visible(),
            None => false,
        }
    }

    pub fn is_char_fully_visible(&self, char_index: usize) -> bool {
        match self.char_visibility.get(char_index) {
            Some(v) => v.is_fully_visible(),
            None => true,
        }
    }

    pub fn get_covered_chars(&self) -> Vec<usize> {
        self.char_visibility
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if !v.is_any_visible() { Some(i) } else { None })
            .collect()
    }

    pub fn get_partially_covered_chars(&self) -> Vec<usize> {
        self.char_visibility
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                if v.is_any_visible() && !v.is_fully_visible() {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn get_visible_chars(&self) -> Vec<usize> {
        self.char_visibility
            .iter()
            .enumerate()
            .filter_map(|(i, v)| if v.is_fully_visible() { Some(i) } else { None })
            .collect()
    }

    pub fn reset(&mut self) {
        self.char_boxes.clear();
        self.char_visibility.clear();
        self.drawing_ops.clear();
    }

    pub fn get_char_box(&self, char_index: usize) -> Option<CharBox> {
        self.char_boxes.get(char_index).copied()
    }

    pub fn get_coverage_summary(&self) -> CoverageSummary {
        let total = self.char_visibility.len() as f64;
        if total == 0.0 {
            return CoverageSummary::default();
        }

        let fully_covered = self.get_covered_chars().len() as f64;
        let partially_covered = self.get_partially_covered_chars().len() as f64;
        let fully_visible = self.get_visible_chars().len() as f64;

        CoverageSummary {
            total_chars: self.char_visibility.len(),
            fully_covered_count: fully_covered as usize,
            partially_covered_count: partially_covered as usize,
            fully_visible_count: fully_visible as usize,
            fully_covered_ratio: fully_covered / total,
            partially_covered_ratio: partially_covered / total,
            fully_visible_ratio: fully_visible / total,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CoverageSummary {
    pub total_chars: usize,
    pub fully_covered_count: usize,
    pub partially_covered_count: usize,
    pub fully_visible_count: usize,
    pub fully_covered_ratio: f64,
    pub partially_covered_ratio: f64,
    pub fully_visible_ratio: f64,
}

impl Default for CoveredTextDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_box_creation() {
        let box_ = CharBox::new(10.0, 20.0, 30.0, 50.0);
        assert_eq!(box_.x_min, 10.0);
        assert_eq!(box_.y_min, 20.0);
        assert_eq!(box_.x_max, 30.0);
        assert_eq!(box_.y_max, 50.0);
    }

    #[test]
    fn test_corner_visibility() {
        let all_visible = CornerVisibility::all_visible();
        assert!(all_visible.is_fully_visible());
        assert!(all_visible.is_any_visible());

        let all_covered = CornerVisibility::all_covered();
        assert!(!all_covered.is_fully_visible());
        assert!(!all_covered.is_any_visible());
    }

    #[test]
    fn test_single_char_coverage() {
        let mut detector = CoveredTextDetector::new();

        // Add a character
        detector.add_character(0.0, 0.0, 10.0, 10.0);
        assert!(detector.is_char_fully_visible(0));

        // Add a covering rectangle
        detector.add_non_character(0.0, 0.0, 10.0, 10.0);
        assert!(detector.is_char_fully_covered(0));
    }

    #[test]
    fn test_partial_coverage() {
        let mut detector = CoveredTextDetector::new();

        // Add a character
        detector.add_character(0.0, 0.0, 20.0, 20.0);

        // Add a partially covering rectangle
        detector.add_non_character(0.0, 0.0, 10.0, 10.0);

        assert!(detector.is_char_partially_visible(0));
        assert!(!detector.is_char_fully_visible(0));
        assert!(!detector.is_char_fully_covered(0));
    }

    #[test]
    fn test_multiple_chars() {
        let mut detector = CoveredTextDetector::new();

        // Add multiple characters
        detector.add_character(0.0, 0.0, 10.0, 10.0);
        detector.add_character(20.0, 0.0, 30.0, 10.0);
        detector.add_character(0.0, 20.0, 10.0, 30.0);

        // Cover only the first character
        detector.add_non_character(0.0, 0.0, 10.0, 10.0);

        assert!(detector.is_char_fully_covered(0));
        assert!(detector.is_char_fully_visible(1));
        assert!(detector.is_char_fully_visible(2));

        let summary = detector.get_coverage_summary();
        assert_eq!(summary.fully_covered_count, 1);
        assert_eq!(summary.fully_visible_count, 2);
    }

    #[test]
    fn test_no_overlap() {
        let mut detector = CoveredTextDetector::new();

        detector.add_character(0.0, 0.0, 10.0, 10.0);
        detector.add_non_character(20.0, 20.0, 30.0, 30.0);

        assert!(detector.is_char_fully_visible(0));
    }

    #[test]
    fn test_corner_intersection() {
        let mut detector = CoveredTextDetector::new();

        // Add a character (corners: bottom_left at (0,0), bottom_right at (20,0), top_left at (0,20), top_right at (20,20))
        detector.add_character(0.0, 0.0, 20.0, 20.0);

        // Cover bottom-left corner with small rectangle
        detector.add_non_character(0.0, 0.0, 5.0, 5.0);

        let visibility = detector.get_visibility(0).unwrap();
        assert!(!visibility.bottom_left); // bottom_left corner is covered
        assert!(visibility.top_left);
        assert!(visibility.top_right);
        assert!(visibility.bottom_right);
    }
}
