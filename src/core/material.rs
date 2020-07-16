use ash::version::DeviceV1_0;
use ash::{vk, Device};
use crate::renderer::{MainPass, RenderState, Texture};
use std::ptr;
use std::rc::Rc;

pub struct Material
{
	descriptor_sets: Vec<vk::DescriptorSet>,
	texture: Texture,
	normal_map: Texture,

	// Keep a pointer to the device for cleanup
	device: Rc<Device>,
}

impl Material
{
	pub fn new(rs: &RenderState, mp: &MainPass, texture_path: &str, normalmap_path: &str) -> Rc<Material>
	{
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

		let material = Material {
			descriptor_sets: descriptor_sets,
			texture: texture,
			normal_map: normal_map,
			device: Rc::clone(&rs.device),
		};
		// Since materials are generally shared, return a refcount.
		return Rc::new(material);
	}

	pub fn bind_descriptor_sets(&self, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout)
	{
		unsafe {
			self.device.cmd_bind_descriptor_sets(
				cmd_buf,
				vk::PipelineBindPoint::GRAPHICS,
				pipeline_layout,
				0,
				&self.descriptor_sets[..],
				&[],
			);
		}
	}
}

impl Drop for Material
{
	fn drop(&mut self)
	{
		// We cannot have the last reference to device at this point
		debug_assert!(1 < Rc::strong_count(&self.device));
		self.texture.destroy(&self.device);
		self.normal_map.destroy(&self.device);
	}
}
