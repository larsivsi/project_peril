use crate::core::{Material, Mesh};
use ash::version::DeviceV1_0;
use ash::{vk, Device};
use cgmath::Matrix4;
use std::{mem, slice};

pub trait Drawable
{
	fn get_mesh(&self) -> &Mesh;
	fn get_material(&self) -> &Material;

	fn draw(
		&self, device: &Device, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout,
		model_matrix: &Matrix4<f32>, view_matrix: &Matrix4<f32>, projection_matrix: &Matrix4<f32>,
	)
	{
		let mv_matrix = view_matrix * model_matrix;
		let mvp_matrix = projection_matrix * mv_matrix;
		let matrices = [model_matrix.clone(), mvp_matrix];

		self.get_mesh().bind_buffers(cmd_buf);
		self.get_material().bind_descriptor_sets(cmd_buf, pipeline_layout);

		unsafe {
			let matrices_bytes = slice::from_raw_parts(matrices.as_ptr() as *const u8, mem::size_of_val(&matrices));
			device.cmd_push_constants(cmd_buf, pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, matrices_bytes);
			device.cmd_draw_indexed(cmd_buf, self.get_mesh().get_num_indices(), 1, 0, 0, 1);
		}
	}
}
