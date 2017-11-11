pub trait Drawable {
    fn draw(&self);
    fn get_position(&self) -> (f64, f64, f64);
    fn set_position(&mut self, position: (f64, f64, f64));
    fn get_distance<T: Drawable>(&self, object: T) -> f64;
}

pub struct Cuboid {
    dimensions: (u32, u32, u32),
    position: (f64, f64, f64),
}

impl Cuboid {
    pub fn new(width: u32, height: u32, depth: u32) -> Cuboid {
        Cuboid {
            dimensions: (width, height, depth),
            position: (0.0, 0.0, 0.0),
        }
    }
}

impl Drawable for Cuboid {
    fn draw(&self) {}

    fn get_position(&self) -> (f64, f64, f64) {
        self.position
    }

    fn set_position(&mut self, position: (f64, f64, f64)) {
        self.position = position;
    }

    fn get_distance<T: Drawable>(&self, object: T) -> f64 {
        let (x1, y1, z1) = object.get_position();
        let (x2, y2, z2) = self.position;

        ((x2 - x1).powi(2) + (y2 - y1).powi(2) + (z2 - z1).powi(2)).sqrt()
    }
}
