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
		let mut scene = Scene {
			objects: Vec::new(),
		};

		let cuboid = DrawObject::new_cuboid(rs, mp, Point3::new(1.0, 0.0, -4.0), 2.0, 2.0, 2.0);
		scene.objects.push(cuboid);

		let mut right_wall = DrawObject::new_quad(rs, mp, Point3::new(10.0, 0.0, 0.0), 10.0, 10.0);
		right_wall.rotate(Vector3::new(0.0, 1.0, 0.0), Deg(-90.0));
		scene.objects.push(right_wall);

		let mut left_wall = DrawObject::new_quad(rs, mp, Point3::new(-10.0, 0.0, 0.0), 10.0, 10.0);
		left_wall.rotate(Vector3::new(0.0, 1.0, 0.0), Deg(90.0));
		left_wall.rotate(Vector3::new(1.0, 0.0, 0.0), Deg(180.0));
		scene.objects.push(left_wall);

		let mut floor = DrawObject::new_quad(rs, mp, Point3::new(0.0, -10.0, 0.0), 10.0, 10.0);
		floor.rotate(Vector3::new(1.0, 0.0, 0.0), Deg(-90.0));
		floor.rotate(Vector3::new(0.0, 1.0, 0.0), Deg(-90.0));
		scene.objects.push(floor);

		scene
	}

	pub fn update(&mut self)
	{
		for (i, mut object) in self.objects.iter_mut().enumerate()
		{
			if i > 0
			{
				break;
			}
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
