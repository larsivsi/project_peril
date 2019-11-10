use ash::{vk, Device};
use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Quaternion, Vector3};
use object::transform::Transformable;
use object::{DrawObject, Drawable, Material, Mesh};
use renderer::{MainPass, RenderState};

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

		let quad_mesh = Mesh::new_quad(rs, 20.0, 20.0);
		let cuboid_mesh = Mesh::new_cuboid(rs, 2.0, 2.0, 2.0);
		let metal_panel_surface = Material::new(
			rs,
			mp,
			"assets/thirdparty/textures/Metal_Panel_004/Metal_Panel_004_COLOR.jpg",
			"assets/thirdparty/textures/Metal_Panel_004/Metal_Panel_004_NORM.jpg",
		);
		let cube_surface = Material::new(
			rs,
			mp,
			"assets/original/textures/cubemap.png",
			"assets/original/textures/cubemap_normals.png",
		);
		let cuboid = DrawObject::new(Point3::new(0.0, 0.0, -4.0), cuboid_mesh, cube_surface);
		scene.objects.push(cuboid);

		let points = vec![
			Point3::new(1.0, 0.0, 0.0),
			Point3::new(-1.0, 0.0, 0.0),
			Point3::new(0.0, 1.0, 0.0),
			Point3::new(0.0, -1.0, 0.0),
			Point3::new(0.0, 0.0, -1.0),
			Point3::new(0.0, 0.0, 1.0),
		];

		let directions = vec![
			Vector3::new(0.0, -1.0, 0.0),
			Vector3::new(0.0, 1.0, 0.0),
			Vector3::new(1.0, 0.0, 0.0),
			Vector3::new(-1.0, 0.0, 0.0),
			Vector3::new(0.0, 0.0, 1.0),
			Vector3::new(0.0, 0.0, 1.0),
		];

		for i in 0..6
		{
			let x: f32 = points[i].x;
			let y: f32 = points[i].y;
			let z: f32 = points[i].z;
			let mut wall = DrawObject::new(Point3::new(0., 0., 0.), quad_mesh.clone(), metal_panel_surface.clone());
			wall.globally_rotate(Quaternion::from_axis_angle(directions[i], Deg(90.0)));
			if i == 5
			{
				wall.globally_rotate(Quaternion::new(0.0, 0.0, 1.0, 0.0));
			}
			wall.set_position(Point3::new(20. * x, 20. * y, 20. * z));
			scene.objects.push(wall);
		}

		return scene;
	}

	pub fn update(&mut self)
	{
		for (i, object) in self.objects.iter_mut().enumerate()
		{
			if i > 0
			{
				break;
			}
			// TODO: Move this.
			object.globally_rotate(Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Deg(-0.5)));
			object.scale(1.001);
		}
	}

	pub fn draw(
		&self, device: &Device, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout,
		view_matrix: &Matrix4<f32>, projection_matrix: &Matrix4<f32>,
	)
	{
		for object in self.objects.iter()
		{
			object.draw(device, cmd_buf, pipeline_layout, view_matrix, projection_matrix);
		}
	}
}
