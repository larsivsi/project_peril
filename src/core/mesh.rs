use ash::version::DeviceV1_0;
use ash::{vk, Device};
use renderer::RenderState;
use std::rc::Rc;

// We never read the fields explicitly, hence they're counted as dead code.
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Vertex
{
	pos: [f32; 3],
	normal: [f32; 3],
	tangent: [f32; 3],
	bitangent: [f32; 3],
	tex_uv: [f32; 2],
}

pub struct Mesh
{
	vertices: vk::Buffer,
	vertex_mem: vk::DeviceMemory,
	indices: vk::Buffer,
	index_mem: vk::DeviceMemory,
	num_indices: u32,

	// Keep a pointer to the device for cleanup
	device: Rc<Device>,
}

impl Mesh
{
	fn new(rs: &RenderState, vertices: &[Vertex], indices: &[u16]) -> Rc<Mesh>
	{
		// Create buffer for vertices
		let (vert_buffer, vert_mem) = rs.create_buffer_and_upload(
			vk::BufferUsageFlags::VERTEX_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL,
			&vertices,
			true,
		);

		// Create buffer for indices
		let (idx_buffer, idx_mem) = rs.create_buffer_and_upload(
			vk::BufferUsageFlags::INDEX_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL,
			&indices,
			true,
		);

		let mesh = Mesh {
			vertices: vert_buffer,
			vertex_mem: vert_mem,
			indices: idx_buffer,
			index_mem: idx_mem,
			num_indices: indices.len() as u32,
			device: Rc::clone(&rs.device),
		};
		// Since materials are generally shared, return a refcount.
		return Rc::new(mesh);
	}

	pub fn bind_buffers(&self, cmd_buf: vk::CommandBuffer)
	{
		unsafe {
			self.device.cmd_bind_vertex_buffers(cmd_buf, 0, &[self.vertices], &[0]);
			self.device.cmd_bind_index_buffer(cmd_buf, self.indices, 0, vk::IndexType::UINT16);
		}
	}

	pub fn get_num_indices(&self) -> u32
	{
		return self.num_indices;
	}

	pub fn new_quad(rs: &RenderState, width: f32, height: f32) -> Rc<Mesh>
	{
		let vertices = [
			Vertex {
				pos: [-width, -height, 0.0],
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.0, 0.0],
			},
			Vertex {
				pos: [width, -height, 0.0],
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [1.0, 0.0],
			},
			Vertex {
				pos: [-width, height, 0.0],
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.0, 1.0],
			},
			Vertex {
				pos: [width, height, 0.0],
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [1.0, 1.0],
			},
		];
		let indices = [0u16, 1, 3, 0, 3, 2];

		return Mesh::new(rs, &vertices, &indices);
	}

	pub fn new_cuboid(rs: &RenderState, width: f32, height: f32, depth: f32) -> Rc<Mesh>
	{
		let half_width = width / 2.0;
		let half_height = height / 2.0;
		let half_depth = depth / 2.0;

		let vertices = [
			// Front
			Vertex {
				pos: [-half_width, -half_height, half_depth], // Lower Left Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.25, 2.0 / 3.0],
			},
			Vertex {
				pos: [half_width, -half_height, half_depth], // Lower Right Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 2.0 / 3.0],
			},
			Vertex {
				pos: [-half_width, half_height, half_depth], // Upper Left Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.25, 1.0 / 3.0],
			},
			Vertex {
				pos: [half_width, half_height, half_depth], // Upper Right Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 1.0 / 3.0],
			},
			// Back
			Vertex {
				pos: [half_width, -half_height, -half_depth], // Lower Right Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 2.0 / 3.0],
			},
			Vertex {
				pos: [-half_width, -half_height, -half_depth], // Lower Left Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [1.0, 2.0 / 3.0],
			},
			Vertex {
				pos: [half_width, half_height, -half_depth], // Upper Right Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 1.0 / 3.0],
			},
			Vertex {
				pos: [-half_width, half_height, -half_depth], // Upper Left Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [1.0, 1.0 / 3.0],
			},
			// Top
			Vertex {
				pos: [-half_width, half_height, half_depth], // Upper Left Front
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.25, 1.0 / 3.0],
			},
			Vertex {
				pos: [half_width, half_height, half_depth], // Upper Right Front
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.5, 1.0 / 3.0],
			},
			Vertex {
				pos: [-half_width, half_height, -half_depth], // Upper Left Back
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.25, 0.0],
			},
			Vertex {
				pos: [half_width, half_height, -half_depth], // Upper Right Back
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.5, 0.0],
			},
			// Bottom
			Vertex {
				pos: [-half_width, -half_height, -half_depth], // Lower Left Back
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.25, 1.0],
			},
			Vertex {
				pos: [half_width, -half_height, -half_depth], // Lower Right Back
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.5, 1.0],
			},
			Vertex {
				pos: [-half_width, -half_height, half_depth], // Lower Left Front
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.25, 2.0 / 3.0],
			},
			Vertex {
				pos: [half_width, -half_height, half_depth], // Lower Right Front
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.5, 2.0 / 3.0],
			},
			// Right
			Vertex {
				pos: [half_width, -half_height, half_depth], // Lower Right Front
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 2.0 / 3.0],
			},
			Vertex {
				pos: [half_width, -half_height, -half_depth], // Lower Right Back
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 2.0 / 3.0],
			},
			Vertex {
				pos: [half_width, half_height, half_depth], // Upper Right Front
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 1.0 / 3.0],
			},
			Vertex {
				pos: [half_width, half_height, -half_depth], // Upper Right Back
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 1.0 / 3.0],
			},
			// Left
			Vertex {
				pos: [-half_width, -half_height, -half_depth], // Lower Left Back
				normal: [-1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, 1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.0, 2.0 / 3.0],
			},
			Vertex {
				pos: [-half_width, -half_height, half_depth], // Lower Left Front
				normal: [-1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, 1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.25, 2.0 / 3.0],
			},
			Vertex {
				pos: [-half_width, half_height, -half_depth], // Upper Left Back
				normal: [-1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, 1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.0, 1.0 / 3.0],
			},
			Vertex {
				pos: [-half_width, half_height, half_depth], // Upper Left Front
				normal: [-1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, 1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.25, 1.0 / 3.0],
			},
		];
		let indices = [
			0u16, 1, 2, 2, 1, 3, // Front
			4, 5, 6, 6, 5, 7, // Back
			8, 9, 10, 10, 9, 11, // Top
			12, 13, 14, 14, 13, 15, // Bottom
			16, 17, 18, 18, 17, 19, // Right
			20, 21, 22, 22, 21, 23, // Left
		];

		return Mesh::new(rs, &vertices, &indices);
	}
}

impl Drop for Mesh
{
	fn drop(&mut self)
	{
		// We cannot have the last reference to device at this point
		debug_assert!(1 < Rc::strong_count(&self.device));

		unsafe {
			self.device.destroy_buffer(self.indices, None);
			self.device.free_memory(self.index_mem, None);
			self.device.destroy_buffer(self.vertices, None);
			self.device.free_memory(self.vertex_mem, None);
		}
	}
}
