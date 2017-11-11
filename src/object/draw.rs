use object::{Drawable, Position};

#[derive(Debug)]
pub struct DrawObject {
    position: (f64, f64, f64),
}

impl Drawable for DrawObject {
    fn draw(&self) {}
}

impl Position for DrawObject {
    fn new(position: (f64, f64, f64)) -> DrawObject {
        DrawObject { position: position }
    }

    fn get_position(&self) -> (f64, f64, f64) {
        self.position
    }

    fn set_position(&mut self, position: (f64, f64, f64)) {
        self.position = position;
    }

    fn get_distance<T: Position>(&self, object: T) -> f64 {
        let (x1, y1, z1) = object.get_position();
        let (x2, y2, z2) = self.position;

        ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt()
    }
}
