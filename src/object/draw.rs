use cgmath::Point3;
use object::{Drawable, Position};

#[derive(Debug)]
pub struct DrawObject {
    position: Point3<f64>,
}

impl Drawable for DrawObject {
    fn draw(&self) {}
}

impl Position for DrawObject {
    fn new(position: Point3<f64>) -> DrawObject {
        DrawObject { position: position }
    }

    fn get_position(&self) -> Point3<f64> {
        self.position
    }

    fn set_position(&mut self, position: Point3<f64>) {
        self.position = position;
    }
}
