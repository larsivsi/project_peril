mod camera;
mod draw;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

use cgmath::prelude::*;
use cgmath::Point3;

pub trait Drawable {
    /// Draws the given object.
    fn draw(&self);
}

pub trait Position {
    /// Returns the position of the given object.
    fn get_position(&self) -> Point3<f64>;

    /// Sets the position of the given object.
    fn set_position(&mut self, position: Point3<f64>);

    /// Gets the distance between the given and passed objects.
    fn get_distance<T: Position>(&self, other: &T) -> f64 {
        let vec = other.get_position() - self.get_position();
        vec.dot(vec).sqrt()
    }
}
