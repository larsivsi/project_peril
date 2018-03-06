use cgmath::prelude::*;
use cgmath::{Matrix4, Point3, Vector3};
use object::{Moveable, Position};

pub struct Camera {
    position: Point3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>,
    world_up: Vector3<f32>,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    /// Updates the front, right and up-vectors based on the camera's pitch and yaw.
    fn update(&mut self) {
        self.front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        self.front.y = self.pitch.to_radians().sin();
        self.front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.front.normalize();
        self.right = self.front.cross(self.world_up);
        self.right.normalize();
        self.up = self.right.cross(self.front);
        self.up.normalize();
    }

    /// Creates a new Camera struct
    pub fn new(position: Point3<f32>) -> Camera {
        let mut camera = Camera {
            position: position,
            front: Vector3 {
                x: 0.0,
                y: 0.0,
                z: -1.0,
            },
            up: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            right: Vector3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            world_up: Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            yaw: 270.0,
            pitch: 0.0,
        };
        camera.update();
        camera
    }

    pub fn generate_view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_dir(self.position, self.front, self.up)
    }
}

impl Position for Camera {
    fn get_position(&self) -> Point3<f32> {
        self.position
    }

    fn set_position(&mut self, position: Point3<f32>) {
        self.position = position;
    }
}

impl Moveable for Camera {
    fn translate(&mut self, direction: Vector3<f32>) {
        let camera_movement_speed = 1;
        self.position += direction;
    }
}
