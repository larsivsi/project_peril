use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Quaternion, Rad, Vector3};
use object::{Position, Rotation};

pub struct Camera
{
	position: Point3<f32>,
	initial_front: Vector3<f32>,
	rotation: Quaternion<f32>,
	front: Vector3<f32>,
	right: Vector3<f32>,
	up: Vector3<f32>,
}

impl Camera
{
	/// Updates the front, right and up-vectors based on the camera's pitch and yaw.
	fn update(&mut self)
	{
		let world_up = Vector3::unit_y();
		self.front = self.get_front_vector();
		self.right = self.front.cross(world_up).normalize();
		self.up = self.right.cross(self.front).normalize();
	}

	/// Creates a new Camera struct
	pub fn new(position: Point3<f32>) -> Camera
	{
		let mut camera = Camera {
			position: position,
			initial_front: Vector3 {
				x: 0.0,
				y: 0.0,
				z: -1.0,
			},
			// Initially no rotation
			rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
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

	/// Visit https://gamedev.stackexchange.com/a/136175 for a good explanation of this
	pub fn yaw(&mut self, angle: f32)
	{
		let yaw = Quaternion::from_axis_angle(Vector3::unit_y(), Deg(angle));
		// global yaw
		self.globally_rotate(yaw);
		self.update();
	}

	pub fn pitch(&mut self, angle: f32)
	{
		let world_up = Vector3::unit_y();
		// Ignore if camera would point to directly up
		if angle > 0.0 && self.front.angle(world_up) <= Rad::from(Deg(angle))
		{
			return;
		}
		// Ignore if camera would point directly down
		else if angle < 0.0 && self.front.angle(world_up * -1.0) <= Rad::from(Deg(angle.abs()))
		{
			return;
		}
		let pitch = Quaternion::from_axis_angle(Vector3::unit_x(), Deg(angle));
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
	fn get_initial_front(&self) -> Vector3<f32>
	{
		self.initial_front
	}

	fn get_rotation(&self) -> Quaternion<f32>
	{
		self.rotation
	}

	fn set_rotation(&mut self, rotation: Quaternion<f32>)
	{
		self.rotation = rotation;
	}
}
