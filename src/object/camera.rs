use vector::Vec3;
use object::Position;

pub struct Camera {
    position: (f64, f64, f64),
    front: Vec3,
    up: Vec3,
    right: Vec3,
    world_up: Vec3,
    yaw: f64,
    pitch: f64,
}

impl Camera {
    fn update(&mut self) {
        self.front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.front.y = self.pitch.to_radians().sin();
        self.front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.front.normalize();
        self.right = self.front.cross(&self.world_up);
        self.right.normalize();
        self.up = self.right.cross(&self.front);
        self.up.normalize();
    }
}

impl Position for Camera {
    fn new(position: (f64, f64, f64)) -> Camera {
        let mut camera = Camera {
            position: position,
            front: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            right: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            world_up: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            yaw: 0.0,
            pitch: 0.0,
        };
        camera.update();
        camera
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
