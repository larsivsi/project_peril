use ash::version::DeviceV1_0;
use ash::{vk, Device};
use cgmath::{Matrix4, Point3};
use object::transform::{Transform, Transformable};
use object::{Drawable, Material, Mesh};
use std::rc::Rc;
use std::{mem, slice};

pub struct DrawObject
{
	transform: Transform,
	mesh: Rc<Mesh>,
	material: Rc<Material>,
}

impl Drawable for DrawObject
{
	fn get_mesh(&self) -> &Mesh
	{
		return &self.mesh;
	}
	fn get_material(&self) -> &Material
	{
		return &self.material;
	}

	fn draw(
		&self, device: &Device, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout,
		view_matrix: &Matrix4<f32>, projection_matrix: &Matrix4<f32>,
	)
	{
		let model_matrix = self.generate_transformation_matrix();
		let mv_matrix = view_matrix * model_matrix;
		let mvp_matrix = projection_matrix * mv_matrix;
		let matrices = [model_matrix, mvp_matrix];

		self.get_mesh().bind_buffers(cmd_buf);
		self.get_material().bind_descriptor_sets(cmd_buf, pipeline_layout);

		unsafe {
			let matrices_bytes = slice::from_raw_parts(matrices.as_ptr() as *const u8, mem::size_of_val(&matrices));
			device.cmd_push_constants(cmd_buf, pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, matrices_bytes);
			device.cmd_draw_indexed(cmd_buf, self.get_mesh().get_num_indices(), 1, 0, 0, 1);
		}
	}
}

impl Transformable for DrawObject
{
	fn get_transform(&self) -> &Transform
	{
		return &self.transform;
	}

	fn get_mutable_transform(&mut self) -> &mut Transform
	{
		return &mut self.transform;
	}
}

impl DrawObject
{
	pub fn new(position: Point3<f32>, mesh: Rc<Mesh>, material: Rc<Material>) -> DrawObject
	{
		let mut object = DrawObject {
			transform: Transform::new(),
			mesh: mesh,
			material: material,
		};
		object.set_position(position);

		return object;
	}
}
