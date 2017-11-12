mod camera;
mod draw;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

use cgmath::Point3;

pub trait Drawable {
    fn draw(&self);
}

pub trait Position {
    fn new(postion: Point3<f64>) -> Self;
    fn get_position(&self) -> Point3<f64>;
    fn set_position(&mut self, position: Point3<f64>);
    fn get_distance<T: Position>(&self, object: T) -> f64 {
        let Point3 {
            x: x1,
            y: y1,
            z: z1,
        } = object.get_position();
        let Point3 {
            x: x2,
            y: y2,
            z: z2,
        } = self.get_position();

        ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt()
    }
}
