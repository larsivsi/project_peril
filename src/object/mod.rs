mod camera;
pub mod draw;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

use ash::vk;
use cgmath::prelude::*;
use cgmath::{Euler, Matrix4, Point3, Quaternion, Vector3};

pub trait Drawable
{
	/// Draws the given object.
	fn draw(
		&self, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout, view_matrix: &Matrix4<f32>,
		projection_matrix: &Matrix4<f32>,
	);
}

pub trait Position
{
	/// Returns the position of the given object.
	fn get_position(&self) -> Point3<f32>;

	/// Sets the position of the given object.
	fn set_position(&mut self, position: Point3<f32>);

	/// Gets the distance between the given and passed objects.
	fn get_distance<T: Position>(&self, other: &T) -> f32
	{
		let vec = other.get_position() - self.get_position();
		vec.dot(vec).sqrt()
	}

	/// Translates the object
	fn translate(&mut self, translation: Vector3<f32>)
	{
		let mut position = self.get_position();
		position += translation;
		self.set_position(position);
	}
}

pub trait Rotation
{
	fn get_rotation(&self) -> Quaternion<f32>;
	fn set_rotation(&mut self, rotation: Quaternion<f32>);

	/// Visit https://gamedev.stackexchange.com/a/136175 for a good explanation of this
	fn globally_rotate(&mut self, rotation: Quaternion<f32>)
	{
		let cur_rotation = self.get_rotation();
		// global rotation, notice the order
		let new_rotation = rotation * cur_rotation;
		self.set_rotation(new_rotation);
	}
	fn locally_rotate(&mut self, rotation: Quaternion<f32>)
	{
		let cur_rotation = self.get_rotation();
		// local rotation, notice the order
		let new_rotation = cur_rotation * rotation;
		self.set_rotation(new_rotation);
	}

	fn get_front_vector(&self) -> Vector3<f32>
	{
		let cur_rotation = self.get_rotation();

		let euler_angles = Euler::from(cur_rotation);
		let yaw = euler_angles.x;
		let pitch = euler_angles.y;

		let front = Vector3::new(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());

		front.normalize()
	}
}
