use ash::version::DeviceV1_0;
use ash::vk;
use ash::Device;
use cgmath::{Matrix4, Point3, Vector3};
use object::transform::{Transform, Transformable};
use object::Drawable;
use renderer::{MainPass, RenderState, Texture};
use std::rc::Rc;
use std::{f32, mem, ptr, slice};

#[derive(Clone, Copy)]
#[allow(dead_code)] // not going to change vertices after creation
pub struct Vertex
{
	pos: [f32; 3],
	normal: [f32; 3],
	tangent: [f32; 3],
	bitangent: [f32; 3],
	tex_uv: [f32; 2],
}

pub struct DrawObject
{
	vertices: vk::Buffer,
	vertex_mem: vk::DeviceMemory,
	indices: vk::Buffer,
	index_mem: vk::DeviceMemory,
	num_indices: u32,

	transform: Transform,

	descriptor_sets: Vec<vk::DescriptorSet>,
	texture: Texture,
	normal_map: Texture,

	// Keep a pointer to the device for cleanup
	device: Rc<Device>,
}

impl Drawable for DrawObject
{
	fn draw(
		&self, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout, view_matrix: &Matrix4<f32>,
		projection_matrix: &Matrix4<f32>,
	)
	{
		let model_matrix = self.generate_transformation_matrix();
		let mv_matrix = view_matrix * model_matrix;
		let mvp_matrix = projection_matrix * mv_matrix;
		let matrices = [model_matrix, mvp_matrix];

		let matrices_bytes;
		unsafe {
			matrices_bytes = slice::from_raw_parts(matrices.as_ptr() as *const u8, mem::size_of_val(&matrices));
		}

		unsafe {
			self.device.cmd_push_constants(cmd_buf, pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, matrices_bytes);
			self.device.cmd_bind_descriptor_sets(
				cmd_buf,
				vk::PipelineBindPoint::GRAPHICS,
				pipeline_layout,
				0,
				&self.descriptor_sets[..],
				&[],
			);
			self.device.cmd_bind_vertex_buffers(cmd_buf, 0, &[self.vertices], &[0]);
			self.device.cmd_bind_index_buffer(cmd_buf, self.indices, 0, vk::IndexType::UINT16);
			self.device.cmd_draw_indexed(cmd_buf, self.num_indices, 1, 0, 0, 1);
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
	fn new(
		rs: &RenderState, mp: &MainPass, position: Point3<f32>, initial_front: Vector3<f32>, vertices: &[Vertex],
		indices: &[u16], texture_path: &str, normalmap_path: &str,
	) -> DrawObject
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

		let desc_alloc_info = vk::DescriptorSetAllocateInfo {
			s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
			p_next: ptr::null(),
			descriptor_pool: mp.descriptor_pool,
			descriptor_set_count: 1,
			p_set_layouts: &mp.descriptor_set_layouts[0],
		};
		let descriptor_sets;
		unsafe {
			descriptor_sets = rs.device.allocate_descriptor_sets(&desc_alloc_info).unwrap();
		}

		let texture = rs.load_image(texture_path, true);
		let texture_descriptor = vk::DescriptorImageInfo {
			image_layout: texture.current_layout,
			image_view: texture.view,
			sampler: texture.sampler,
		};

		let normal_map = rs.load_image(normalmap_path, false);
		let normal_descriptor = vk::DescriptorImageInfo {
			image_layout: normal_map.current_layout,
			image_view: normal_map.view,
			sampler: normal_map.sampler,
		};

		let write_desc_sets = [
			vk::WriteDescriptorSet {
				s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
				p_next: ptr::null(),
				dst_set: descriptor_sets[0],
				dst_binding: 0,
				dst_array_element: 0,
				descriptor_count: 1,
				descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
				p_image_info: &texture_descriptor,
				p_buffer_info: ptr::null(),
				p_texel_buffer_view: ptr::null(),
			},
			vk::WriteDescriptorSet {
				s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
				p_next: ptr::null(),
				dst_set: descriptor_sets[0],
				dst_binding: 1,
				dst_array_element: 0,
				descriptor_count: 1,
				descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
				p_image_info: &normal_descriptor,
				p_buffer_info: ptr::null(),
				p_texel_buffer_view: ptr::null(),
			},
		];
		unsafe {
			rs.device.update_descriptor_sets(&write_desc_sets, &[]);
		}

		let mut object = DrawObject {
			vertices: vert_buffer,
			vertex_mem: vert_mem,
			indices: idx_buffer,
			index_mem: idx_mem,
			num_indices: indices.len() as u32,
			transform: Transform::new(),
			descriptor_sets: descriptor_sets,
			texture: texture,
			normal_map: normal_map,
			device: Rc::clone(&rs.device),
		};
		object.set_position(position);
		object.set_initial_front_vector(initial_front);

		return object;
	}

	/// Creates a new quad draw object.
	pub fn new_quad(rs: &RenderState, mp: &MainPass, position: Point3<f32>, width: f32, height: f32) -> DrawObject
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

		DrawObject::new(
			rs,
			mp,
			position,
			Vector3::unit_z(),
			&vertices,
			&indices,
			"assets/thirdparty/textures/Metal_Panel_004/Metal_Panel_004_COLOR.jpg",
			"assets/thirdparty/textures/Metal_Panel_004/Metal_Panel_004_NORM.jpg",
		)
	}

	pub fn new_cuboid(
		rs: &RenderState, mp: &MainPass, position: Point3<f32>, width: f32, height: f32, depth: f32,
	) -> DrawObject
	{
		let h_width = width / 2.0;
		let h_height = height / 2.0;
		let h_depth = depth / 2.0;
		let vertices = [
			// Front
			Vertex {
				pos: [-h_width, -h_height, h_depth], // Lower Left Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.25, 2.0 / 3.0],
			},
			Vertex {
				pos: [h_width, -h_height, h_depth], // Lower Right Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 2.0 / 3.0],
			},
			Vertex {
				pos: [-h_width, h_height, h_depth], // Upper Left Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.25, 1.0 / 3.0],
			},
			Vertex {
				pos: [h_width, h_height, h_depth], // Upper Right Front
				normal: [0.0, 0.0, 1.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 1.0 / 3.0],
			},
			// Back
			Vertex {
				pos: [h_width, -h_height, -h_depth], // Lower Right Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 2.0 / 3.0],
			},
			Vertex {
				pos: [-h_width, -h_height, -h_depth], // Lower Left Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [1.0, 2.0 / 3.0],
			},
			Vertex {
				pos: [h_width, h_height, -h_depth], // Upper Right Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 1.0 / 3.0],
			},
			Vertex {
				pos: [-h_width, h_height, -h_depth], // Upper Left Back
				normal: [0.0, 0.0, -1.0],
				tangent: [-1.0, 0.0, 0.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [1.0, 1.0 / 3.0],
			},
			// Top
			Vertex {
				pos: [-h_width, h_height, h_depth], // Upper Left Front
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.25, 1.0 / 3.0],
			},
			Vertex {
				pos: [h_width, h_height, h_depth], // Upper Right Front
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.5, 1.0 / 3.0],
			},
			Vertex {
				pos: [-h_width, h_height, -h_depth], // Upper Left Back
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.25, 0.0],
			},
			Vertex {
				pos: [h_width, h_height, -h_depth], // Upper Right Back
				normal: [0.0, 1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, -1.0],
				tex_uv: [0.5, 0.0],
			},
			// Bottom
			Vertex {
				pos: [-h_width, -h_height, -h_depth], // Lower Left Back
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.25, 1.0],
			},
			Vertex {
				pos: [h_width, -h_height, -h_depth], // Lower Right Back
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.5, 1.0],
			},
			Vertex {
				pos: [-h_width, -h_height, h_depth], // Lower Left Front
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.25, 2.0 / 3.0],
			},
			Vertex {
				pos: [h_width, -h_height, h_depth], // Lower Right Front
				normal: [0.0, -1.0, 0.0],
				tangent: [1.0, 0.0, 0.0],
				bitangent: [0.0, 0.0, 1.0],
				tex_uv: [0.5, 2.0 / 3.0],
			},
			// Right
			Vertex {
				pos: [h_width, -h_height, h_depth], // Lower Right Front
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 2.0 / 3.0],
			},
			Vertex {
				pos: [h_width, -h_height, -h_depth], // Lower Right Back
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 2.0 / 3.0],
			},
			Vertex {
				pos: [h_width, h_height, h_depth], // Upper Right Front
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.5, 1.0 / 3.0],
			},
			Vertex {
				pos: [h_width, h_height, -h_depth], // Upper Right Back
				normal: [1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, -1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.75, 1.0 / 3.0],
			},
			// Left
			Vertex {
				pos: [-h_width, -h_height, -h_depth], // Lower Left Back
				normal: [-1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, 1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.0, 2.0 / 3.0],
			},
			Vertex {
				pos: [-h_width, -h_height, h_depth], // Lower Left Front
				normal: [-1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, 1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.25, 2.0 / 3.0],
			},
			Vertex {
				pos: [-h_width, h_height, -h_depth], // Upper Left Back
				normal: [-1.0, 0.0, 0.0],
				tangent: [0.0, 0.0, 1.0],
				bitangent: [0.0, 1.0, 0.0],
				tex_uv: [0.0, 1.0 / 3.0],
			},
			Vertex {
				pos: [-h_width, h_height, h_depth], // Upper Left Front
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

		DrawObject::new(
			rs,
			mp,
			position,
			Vector3::unit_z(),
			&vertices,
			&indices,
			"assets/original/textures/cubemap.png",
			"assets/original/textures/cubemap_normals.png",
		)
	}
}

impl Drop for DrawObject
{
	/// Drops the DrawObject by freeing the index and vertex buffers.
	fn drop(&mut self)
	{
		// We cannot have the last reference to device at this point
		debug_assert!(1 < Rc::strong_count(&self.device));

		unsafe {
			self.device.destroy_sampler(self.normal_map.sampler, None);
			self.device.destroy_image_view(self.normal_map.view, None);
			self.device.destroy_image(self.normal_map.image, None);
			self.device.free_memory(self.normal_map.memory, None);

			self.device.destroy_sampler(self.texture.sampler, None);
			self.device.destroy_image_view(self.texture.view, None);
			self.device.destroy_image(self.texture.image, None);
			self.device.free_memory(self.texture.memory, None);

			self.device.destroy_buffer(self.indices, None);
			self.device.free_memory(self.index_mem, None);
			self.device.destroy_buffer(self.vertices, None);
			self.device.free_memory(self.vertex_mem, None);
		}
	}
}
