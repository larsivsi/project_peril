mod camera;
pub mod draw;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

use ash::vk;
use cgmath::{Deg, Matrix4, Point3, Quaternion, Vector3};
use cgmath::prelude::*;

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

	fn rotate(&mut self, axis: Vector3<f32>, angle: Deg<f32>)
	{
		let rotation = self.get_rotation();
		// The order here is important
		let new_rotation = Quaternion::from_axis_angle(axis, angle) * rotation;
		self.set_rotation(new_rotation);
	}
}
