use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Quaternion, Rad, Vector3};

pub trait Transformable
{
	fn get_transform(&self) -> &Transform;
	fn get_mutable_transform(&mut self) -> &mut Transform;

	fn get_front_vector(&self) -> Vector3<f32>
	{
		return self.get_transform().get_front_vector();
	}

	fn get_right_vector(&self) -> Vector3<f32>
	{
		return self.get_transform().get_right_vector();
	}

	fn set_position(&mut self, position: Point3<f32>)
	{
		self.get_mutable_transform().set_position(position);
	}

	fn set_initial_front_vector(&mut self, initial_front: Vector3<f32>)
	{
		self.get_mutable_transform().set_initial_front_vector(initial_front);
	}

	fn translate(&mut self, translation: Vector3<f32>)
	{
		self.get_mutable_transform().translate(translation);
	}

	fn globally_rotate(&mut self, rotation: Quaternion<f32>)
	{
		self.get_mutable_transform().globally_rotate(rotation);
	}

	fn yaw(&mut self, angle: f32)
	{
		self.get_mutable_transform().yaw(angle);
	}

	fn pitch(&mut self, angle: f32)
	{
		self.get_mutable_transform().pitch(angle);
	}

	fn generate_transformation_matrix(&self) -> Matrix4<f32>
	{
		return self.get_transform().generate_transformation_matrix();
	}

	fn generate_view_matrix(&self) -> Matrix4<f32>
	{
		return self.get_transform().generate_view_matrix();
	}
}

pub struct Transform
{
	position: Point3<f32>,
	initial_front: Vector3<f32>,
	rotation: Quaternion<f32>,
	scale: f32,
}

fn get_world_up() -> Vector3<f32>
{
	return Vector3::unit_y();
}

impl Transform
{
	pub fn new() -> Transform
	{
		return Transform {
			position: Point3::new(0.0, 0.0, 0.0),
			initial_front: Vector3::unit_z(),
			rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
			scale: 1.0,
		};
	}

	fn get_position(&self) -> Point3<f32>
	{
		return self.position;
	}

	fn set_position(&mut self, position: Point3<f32>)
	{
		self.position = position;
	}

	fn translate(&mut self, translation: Vector3<f32>)
	{
		let mut position = self.get_position();
		position += translation;
		self.set_position(position);
	}

	fn get_initial_front_vector(&self) -> Vector3<f32>
	{
		return self.initial_front;
	}

	fn set_initial_front_vector(&mut self, initial_front: Vector3<f32>)
	{
		self.initial_front = initial_front;
	}

	fn get_rotation(&self) -> Quaternion<f32>
	{
		return self.rotation;
	}

	fn set_rotation(&mut self, rotation: Quaternion<f32>)
	{
		self.rotation = rotation;
	}

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
		let front = self.get_rotation() * self.get_initial_front_vector();
		return front.normalize();
	}

	fn get_right_vector(&self) -> Vector3<f32>
	{
		let world_up = get_world_up();
		let front = self.get_front_vector();
		return front.cross(world_up).normalize();
	}

	fn yaw(&mut self, angle: f32)
	{
		let yaw = Quaternion::from_axis_angle(Vector3::unit_y(), Deg(angle));
		// global yaw
		self.globally_rotate(yaw);
	}

	fn pitch(&mut self, angle: f32)
	{
		let world_up = get_world_up();
		let front = self.get_front_vector();
		// Ignore if camera would point to directly up
		if angle > 0.0 && front.angle(world_up) <= Rad::from(Deg(angle))
		{
			return;
		}
		// Ignore if camera would point directly down
		else if angle < 0.0 && front.angle(world_up * -1.0) <= Rad::from(Deg(angle.abs()))
		{
			return;
		}
		let pitch = Quaternion::from_axis_angle(Vector3::unit_x(), Deg(angle));
		// local pitch
		self.locally_rotate(pitch);
	}

	fn get_scale(&self) -> f32
	{
		return self.scale;
	}

	fn set_scale(&mut self, scale: f32)
	{
		self.scale = scale;
	}

	fn generate_transformation_matrix(&self) -> Matrix4<f32>
	{
		let translation_matrix = Matrix4::from_translation(self.get_position() - Point3::new(0.0, 0.0, 0.0));
		let rotation_matrix = Matrix4::from(self.rotation);
		let scale_matrix = Matrix4::from_scale(self.scale);

		let transform_matrix = translation_matrix * rotation_matrix * scale_matrix;
		return transform_matrix;
	}

	fn generate_view_matrix(&self) -> Matrix4<f32>
	{
		let world_up = get_world_up();
		let front = self.get_front_vector();
		let right = front.cross(world_up).normalize();
		let up = right.cross(front).normalize();

		return Matrix4::look_at_dir(self.position, front, up);
	}
}
