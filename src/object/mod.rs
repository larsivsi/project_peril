mod camera;
mod draw;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

pub trait Drawable {
    fn draw(&self);
}

pub trait Position {
    fn new(postion: (f64, f64, f64)) -> Self;
    fn get_position(&self) -> (f64, f64, f64);
    fn set_position(&mut self, position: (f64, f64, f64));
    fn get_distance<T: Position>(&self, object: T) -> f64;
}
