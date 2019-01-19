use crate::my_strategy::vec2::Vec2;

#[derive(Debug, Clone)]
pub struct Line2 {
    begin: Vec2,
    end: Vec2,
}

impl Line2 {
    pub fn new(begin: Vec2, end: Vec2) -> Self {
        Line2 { begin, end }
    }

    pub fn possible_intersection(&self, other: &Line2) -> Option<Vec2> {
        let dx = Vec2::new(self.begin.x() - self.end.x(), other.begin.x() - other.end.x());
        let dy = Vec2::new(self.begin.y() - self.end.y(), other.begin.y() - other.end.y());
        let div = dx.det(dy);
        if div == 0.0 {
            return None;
        }
        let d = Vec2::new(self.begin.det(self.end), other.begin.det(other.end));
        let x = d.det(dx) / div;
        let y = d.det(dy) / div;
        Some(Vec2::new(x, y))
    }
}
