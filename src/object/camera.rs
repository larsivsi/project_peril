use cgmath::{Deg, Euler, Matrix4, Point3, Quaternion, Vector3};
use cgmath::prelude::*;
use object::{Position, Rotation};

pub struct Camera
{
	position: Point3<f32>,
	rotation: Quaternion<f32>,
	front: Vector3<f32>,
	right: Vector3<f32>,
	up: Vector3<f32>,
	world_up: Vector3<f32>,
}

impl Camera
{
	/// Updates the front, right and up-vectors based on the camera's pitch and yaw.
	fn update(&mut self)
	{
		self.front = self.get_front_vector();
		self.right = self.front.cross(self.world_up);
		self.right.normalize();
		self.up = self.right.cross(self.front);
		self.up.normalize();
	}

	/// Creates a new Camera struct
	pub fn new(position: Point3<f32>) -> Camera
	{
		let mut camera = Camera {
			position: position,
			rotation: Quaternion::from(Euler::new(Deg(-90.0), Deg(0.0), Deg(0.0))),
			// just set zeroes for these, as they will be overwritten
			front: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			right: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			up: Vector3 {
				x: 0.0,
				y: 0.0,
				z: 0.0,
			},
			// this one must be correct
			world_up: Vector3 {
				x: 0.0,
				y: 1.0,
				z: 0.0,
			},
		};
		camera.update();
		camera
	}

	pub fn get_cam_front(&self) -> Vector3<f32>
	{
		return self.front;
	}

	pub fn get_cam_right(&self) -> Vector3<f32>
	{
		return self.right;
	}

	pub fn get_world_up_vector(&self) -> Vector3<f32>
	{
		return self.world_up;
	}

	/// Visit https://gamedev.stackexchange.com/a/136175 for a good explanation of this
	pub fn yaw(&mut self, angle: f32)
	{
		let yaw = Quaternion::from(Euler::new(Deg(angle), Deg(0.0), Deg(0.0)));
		// global yaw
		self.globally_rotate(yaw);
		self.update();
	}

	pub fn pitch(&mut self, angle: f32)
	{
		let pitch = Quaternion::from(Euler::new(Deg(0.0), Deg(angle), Deg(0.0)));
		// local pitch
		self.locally_rotate(pitch);
		self.update();
	}

	pub fn generate_view_matrix(&self) -> Matrix4<f32>
	{
		Matrix4::look_at_dir(self.position, self.front, self.up)
	}
}

impl Position for Camera
{
	fn get_position(&self) -> Point3<f32>
	{
		self.position
	}

	fn set_position(&mut self, position: Point3<f32>)
	{
		self.position = position;
	}
}

impl Rotation for Camera
{
	fn get_rotation(&self) -> Quaternion<f32>
	{
		self.rotation
	}

	fn set_rotation(&mut self, rotation: Quaternion<f32>)
	{
		self.rotation = rotation;
	}
}
