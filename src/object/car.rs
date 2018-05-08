use ash::vk;
use cgmath::{InnerSpace, Matrix4, Point3, Quaternion, Vector3};
use object::{DrawObject, Drawable, Physics, Position, Rotation};
use renderer::{MainPass, RenderState};
use std::time::Duration;

pub struct Car
{
	mass: f32,
	force: Vector3<f32>,
	velocity: Vector3<f32>,

	draw_obj: DrawObject,
}

impl Car
{
	pub fn new(rs: &RenderState, mp: &MainPass, position: Point3<f32>) -> Car
	{
		Car {
			mass: 2000.0,
			force: Vector3::new(0.0, 0.0, 0.0),
			velocity: Vector3::new(0.0, 0.0, 0.0),
			draw_obj: DrawObject::new_cuboid(rs, mp, position, 1.0, 1.0, 2.0),
		}
	}
}

impl Position for Car
{
	fn get_position(&self) -> Point3<f32>
	{
		self.draw_obj.get_position()
	}

	fn set_position(&mut self, position: Point3<f32>)
	{
		self.draw_obj.set_position(position);
	}
}

impl Rotation for Car
{
	fn get_rotation(&self) -> Quaternion<f32>
	{
		self.draw_obj.get_rotation()
	}

	fn set_rotation(&mut self, rotation: Quaternion<f32>)
	{
		self.draw_obj.set_rotation(rotation);
	}
}

impl Drawable for Car
{
	fn draw(
		&self, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout, view_matrix: &Matrix4<f32>,
		projection_matrix: &Matrix4<f32>,
	)
	{
		self.draw_obj.draw(cmd_buf, pipeline_layout, view_matrix, projection_matrix);
	}
}

impl Physics for Car
{
	fn apply_force(&mut self, force_newton: Vector3<f32>)
	{
		self.force += force_newton;
	}

	fn apply_drag(&mut self, drag_const: f32)
	{
		self.force -= drag_const * self.velocity * self.velocity.dot(self.velocity).sqrt();
	}

	fn apply_gravity(&mut self, standard_gravity: f32)
	{
		// hack ground for now
		if self.get_position().y <= -10.0
		{
			self.velocity.y = 0.0;
			return;
		}
		self.force += self.mass * standard_gravity * Vector3::new(0.0, -1.0, 0.0);
	}

	fn update(&mut self, delta_time: Duration)
	{
		let seconds = delta_time.as_secs() as f32;
		let subsecs = delta_time.subsec_nanos() as f32 / 1_000_000_000.0;
		let duration = seconds + subsecs;

		let acceleration = self.force / self.mass;
		self.velocity += acceleration * duration;
		let mut pos = self.get_position();
		pos += self.velocity * duration;
		self.set_position(pos);

		// cleanup
		self.force = Vector3::new(0.0, 0.0, 0.0);
	}
}
