use super::Position;

pub struct Camera {
    position: (f64, f64, f64),
    front: (f64, f64, f64),
    up: (f64, f64, f64),
    right: (f64, f64, f64),
    world_up: (f64, f64, f64),
    yaw: f64,
    pitch: f64,
}

impl Position for Camera {
    fn new(position: (f64, f64, f64)) -> Camera {
        Camera {
            position: position,
            front: (0.0, 0.0, 1.0),
            up: (0.0, 1.0, 0.0),
            right: (0.0, 0.0, 0.0),
            world_up: (0.0, 1.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        }
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
