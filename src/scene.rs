use ash::vk;
use cgmath::{Deg, Matrix4, Point3, Vector3};
use object::{DrawObject, Drawable, Rotation};
use renderer::{MainPass, RenderState};
use std::f32;

pub struct Scene
{
	objects: Vec<DrawObject>,
}

impl Scene
{
	pub fn new(rs: &RenderState, mp: &MainPass) -> Scene
	{
		// let _quad = DrawObject::new_quad(rs, Point3::new(0.0, 0.0, 0.0), 1.0, 1.0);
		let cuboid = DrawObject::new_cuboid(rs, mp, Point3::new(1.0, 0.0, -4.0), 2.0, 2.0, 2.0);

		let mut scene = Scene {
			objects: Vec::new(),
		};

		scene.objects.push(cuboid);

		scene
	}

	pub fn update(&mut self)
	{
		for mut object in self.objects.iter_mut()
		{
			// TODO: Move this.
			let axis = Vector3::new(0.0, 1.0, 0.0);
			let angle = Deg(-0.5);

			object.rotate(axis, angle);
		}
	}

	pub fn draw(
		&self, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout, view_matrix: &Matrix4<f32>,
		projection_matrix: &Matrix4<f32>,
	)
	{
		for object in self.objects.iter()
		{
			object.draw(cmd_buf, pipeline_layout, view_matrix, projection_matrix);
		}
	}
}
