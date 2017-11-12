mod camera;
mod draw;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

use cgmath::prelude::*;
use cgmath::{Point3, Vector3};

pub trait Drawable {
    fn draw(&self);
}

pub trait Position {
    fn new(postion: Point3<f64>) -> Self;
    fn get_position(&self) -> Point3<f64>;
    fn set_position(&mut self, position: Point3<f64>);
    fn get_distance<T: Position>(&self, other: &T) -> f64 {
        let vec = other.get_position() - self.get_position();
        vec.dot(vec).sqrt()
    }
}
