use nalgebra::Vector2;

#[derive(Debug, Copy, Clone)]
pub struct CollisionRectangle {
    position: Vector2<f32>, // The top-left corner of the rectangle
    size: Vector2<f32>,     // The width and height of the rectangle
}

impl CollisionRectangle {
    pub fn new(position: Vector2<f32>, size: Vector2<f32>) -> Self {
        Self { position, size }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let self_right = self.position.x + self.size.x;
        let self_bottom = self.position.y + self.size.y;
        let other_right = other.position.x + other.size.x;
        let other_bottom = other.position.y + other.size.y;

        // Axis-Aligned Bounding Box (AABB) collision detection
        !(self.position.x > other_right ||       // self is completely to the right of other
          other.position.x > self_right ||       // other is completely to the right of self
          self.position.y > other_bottom ||      // self is completely below other
          other.position.y > self_bottom) // other is completely below self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intersecting_rectangles() {
        let rect1 = CollisionRectangle::new(Vector2::new(0.0, 0.0), Vector2::new(10.0, 10.0));
        let rect2 = CollisionRectangle::new(Vector2::new(5.0, 5.0), Vector2::new(10.0, 10.0));

        assert!(rect1.intersects(&rect2));
        assert!(rect2.intersects(&rect1));
    }

    #[test]
    fn test_non_intersecting_rectangles() {
        let rect1 = CollisionRectangle::new(Vector2::new(0.0, 0.0), Vector2::new(10.0, 10.0));
        let rect2 = CollisionRectangle::new(Vector2::new(20.0, 20.0), Vector2::new(5.0, 5.0));

        assert!(!rect1.intersects(&rect2));
        assert!(!rect2.intersects(&rect1));
    }

    #[test]
    fn test_touching_rectangles_no_overlap() {
        let rect1 = CollisionRectangle::new(Vector2::new(0.0, 0.0), Vector2::new(10.0, 10.0));
        let rect2 = CollisionRectangle::new(Vector2::new(10.0, 0.0), Vector2::new(10.0, 10.0));

        assert!(!rect1.intersects(&rect2)); // Touching but no intersection
        assert!(!rect2.intersects(&rect1));
    }

    #[test]
    fn test_contained_rectangle() {
        let rect1 = CollisionRectangle::new(Vector2::new(0.0, 0.0), Vector2::new(10.0, 10.0));
        let rect2 = CollisionRectangle::new(Vector2::new(2.0, 2.0), Vector2::new(5.0, 5.0));

        assert!(rect1.intersects(&rect2)); // rect2 is fully inside rect1
        assert!(rect2.intersects(&rect1)); // rect1 contains rect2
    }

    #[test]
    fn test_identical_rectangles() {
        let rect1 = CollisionRectangle::new(Vector2::new(0.0, 0.0), Vector2::new(10.0, 10.0));
        let rect2 = CollisionRectangle::new(Vector2::new(0.0, 0.0), Vector2::new(10.0, 10.0));

        assert!(rect1.intersects(&rect2)); // Identical rectangles
        assert!(rect2.intersects(&rect1));
    }
}
