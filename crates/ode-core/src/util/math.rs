use std::ops::{Add, Mul};

const EPS: f64 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformMatrix {
    pub a: f64,
    pub b: f64,
    pub c: f64,
    pub d: f64,
    pub e: f64,
    pub f: f64,
}

impl TransformMatrix {
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    pub fn round(&self) -> Self {
        Self {
            a: round(self.a),
            b: round(self.b),
            c: round(self.c),
            d: round(self.d),
            e: round(self.e),
            f: round(self.f),
        }
    }

    pub fn transform_point(&self, x: f64, y: f64) -> (f64, f64) {
        let new_x = self.a * x + self.c * y + self.e;
        let new_y = self.b * x + self.d * y + self.f;
        (new_x, new_y)
    }

    pub fn transform_delta(&self, dx: f64, dy: f64) -> (f64, f64) {
        let new_dx = self.a * dx + self.c * dy;
        let new_dy = self.b * dx + self.d * dy;
        (new_dx, new_dy)
    }
}

fn round(x: f64) -> f64 {
    if x.abs() > EPS {
        x
    } else {
        0.0
    }
}

pub fn equal(x: f64, y: f64) -> bool {
    (x - y).abs() <= EPS
}

pub fn is_positive(x: f64) -> bool {
    x > EPS
}

impl Mul for TransformMatrix {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }
}

impl Add for TransformMatrix {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            a: self.a + other.a,
            b: self.b + other.b,
            c: self.c + other.c,
            d: self.d + other.d,
            e: self.e + other.e,
            f: self.f + other.f,
        }
    }
}

impl Default for TransformMatrix {
    fn default() -> Self {
        Self::identity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl BoundingBox {
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            x0: x0.min(x1),
            y0: y0.min(y1),
            x1: x0.max(x1),
            y1: y0.max(y1),
        }
    }

    pub fn width(&self) -> f64 {
        (self.x1 - self.x0).abs()
    }

    pub fn height(&self) -> f64 {
        (self.y1 - self.y0).abs()
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let result = BoundingBox::new(
            self.x0.max(other.x0),
            self.y0.max(other.y0),
            self.x1.min(other.x1),
            self.y1.min(other.y1),
        );

        if result.x0 >= result.x1 || result.y0 >= result.y1 {
            None
        } else {
            Some(result)
        }
    }

    pub fn transform(&self, tm: &TransformMatrix) -> Self {
        let corners = [
            tm.transform_point(self.x0, self.y0),
            tm.transform_point(self.x1, self.y0),
            tm.transform_point(self.x0, self.y1),
            tm.transform_point(self.x1, self.y1),
        ];

        let min_x = corners
            .iter()
            .map(|(x, _)| *x)
            .fold(f64::INFINITY, f64::min);
        let max_x = corners
            .iter()
            .map(|(x, _)| *x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = corners
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::INFINITY, f64::min);
        let max_y = corners
            .iter()
            .map(|(_, y)| *y)
            .fold(f64::NEG_INFINITY, f64::max);

        Self::new(min_x, min_y, max_x, max_y)
    }
}

pub fn hypot(x: f64, y: f64) -> f64 {
    (x * x + y * y).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_identity() {
        let tm = TransformMatrix::identity();
        let (x, y) = tm.transform_point(10.0, 20.0);
        assert_eq!((x, y), (10.0, 20.0));
    }

    #[test]
    fn test_matrix_multiply() {
        let tm1 = TransformMatrix {
            a: 2.0,
            b: 0.0,
            c: 0.0,
            d: 2.0,
            e: 5.0,
            f: 5.0,
        };
        let tm2 = TransformMatrix {
            a: 1.5,
            b: 0.0,
            c: 0.0,
            d: 1.5,
            e: 10.0,
            f: 10.0,
        };
        let result = tm1 * tm2;
        let (x, y) = result.transform_point(1.0, 1.0);
        assert_eq!((x, y), (28.0, 28.0));
    }

    #[test]
    fn test_equal() {
        assert!(equal(1.0, 1.0));
        assert!(equal(1.0, 1.00009));
        assert!(!equal(1.0, 1.0002));
    }

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox::new(10.0, 20.0, 50.0, 60.0);
        assert_eq!(bbox.width(), 40.0);
        assert_eq!(bbox.height(), 40.0);
    }

    #[test]
    fn test_bounding_box_intersect() {
        let bbox1 = BoundingBox::new(0.0, 0.0, 10.0, 10.0);
        let bbox2 = BoundingBox::new(5.0, 5.0, 15.0, 15.0);
        let result = bbox1.intersect(&bbox2);
        assert!(result.is_some());
        let intersection = result.unwrap();
        assert_eq!(
            (
                intersection.x0,
                intersection.y0,
                intersection.x1,
                intersection.y1
            ),
            (5.0, 5.0, 10.0, 10.0)
        );
    }

    #[test]
    fn test_hypot() {
        let result = hypot(3.0, 4.0);
        assert!((result - 5.0).abs() < 0.001);
    }
}
