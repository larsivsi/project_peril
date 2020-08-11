use crate::core::{Config, Vertex};
use crate::renderer::{RenderState, Texture};
use ash::version::DeviceV1_0;
use ash::vk;
use ash::Device;
use cgmath::Matrix4;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr;
use std::rc::Rc;

pub struct MainPass
{
	renderpass: vk::RenderPass,
	pub descriptor_pool: vk::DescriptorPool,
	pub descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
	pub pipeline_layout: vk::PipelineLayout,
	viewport: vk::Viewport,
	scissor: vk::Rect2D,
	pipeline: vk::Pipeline,
	// one framebuffer/commandbuffer per image
	framebuffer: vk::Framebuffer,
	commandbuffer: vk::CommandBuffer,

	// Image to render to.
	pub render_image: Texture,
	depth_image: Texture,

	view_matrix_ub: vk::Buffer,
	pub view_matrix_ub_mem: vk::DeviceMemory,
	view_matrix_ds: Vec<vk::DescriptorSet>,

	// Keep a pointer to the device for cleanup
	device: Rc<Device>,
}

impl MainPass
{
	/// Creates a main renderpass.
	fn create_renderpass(rs: &RenderState, render_format: vk::Format) -> vk::RenderPass
	{
		// One attachment, color only. Will produce the presentable image.
		let renderpass_attachments = [
			vk::AttachmentDescription {
				format: render_format,
				flags: vk::AttachmentDescriptionFlags::empty(),
				samples: vk::SampleCountFlags::TYPE_1,
				load_op: vk::AttachmentLoadOp::CLEAR,
				store_op: vk::AttachmentStoreOp::STORE,
				stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
				stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
				initial_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
				final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
			},
			vk::AttachmentDescription {
				format: vk::Format::D32_SFLOAT,
				flags: vk::AttachmentDescriptionFlags::empty(),
				samples: vk::SampleCountFlags::TYPE_1,
				load_op: vk::AttachmentLoadOp::CLEAR,
				store_op: vk::AttachmentStoreOp::DONT_CARE,
				stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
				stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
				initial_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
				final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
			},
		];
		let color_attachment_ref = vk::AttachmentReference {
			attachment: 0,
			layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
		};
		let depth_attachment_ref = vk::AttachmentReference {
			attachment: 1,
			layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
		};
		let subpass = vk::SubpassDescription {
			color_attachment_count: 1,
			p_color_attachments: &color_attachment_ref,
			p_depth_stencil_attachment: &depth_attachment_ref,
			pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
			..Default::default()
		};
		let renderpass_create_info = vk::RenderPassCreateInfo {
			s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
			attachment_count: renderpass_attachments.len() as u32,
			p_attachments: renderpass_attachments.as_ptr(),
			subpass_count: 1,
			p_subpasses: &subpass,
			..Default::default()
		};
		let renderpass;
		unsafe {
			renderpass = rs.device.create_render_pass(&renderpass_create_info, None).unwrap();
		}

		renderpass
	}

	/// Creates a pipeline for the renderpass.
	fn create_pipeline(
		rs: &RenderState, render_size: vk::Extent3D, renderpass: vk::RenderPass,
	) -> (vk::DescriptorPool, Vec<vk::DescriptorSetLayout>, vk::PipelineLayout, vk::Viewport, vk::Rect2D, vk::Pipeline)
	{
		// Descriptors
		let descriptor_sizes = [
			vk::DescriptorPoolSize {
				ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
				descriptor_count: 14,
			},
			vk::DescriptorPoolSize {
				ty: vk::DescriptorType::UNIFORM_BUFFER,
				descriptor_count: 1,
			},
		];
		let descriptor_pool_info = vk::DescriptorPoolCreateInfo {
			s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
			pool_size_count: descriptor_sizes.len() as u32,
			p_pool_sizes: descriptor_sizes.as_ptr(),
			max_sets: 8, // TODO figure out how to properly do this
			..Default::default()
		};
		let descriptor_pool;
		unsafe {
			descriptor_pool = rs.device.create_descriptor_pool(&descriptor_pool_info, None).unwrap();
		}
		let color_normal_tex_dsl_bindings = [
			vk::DescriptorSetLayoutBinding {
				binding: 0,
				descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
				descriptor_count: 1,
				stage_flags: vk::ShaderStageFlags::FRAGMENT,
				p_immutable_samplers: ptr::null(),
			},
			vk::DescriptorSetLayoutBinding {
				binding: 1,
				descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
				descriptor_count: 1,
				stage_flags: vk::ShaderStageFlags::FRAGMENT,
				p_immutable_samplers: ptr::null(),
			},
		];
		let view_matrix_dsl_binding = [vk::DescriptorSetLayoutBinding {
			binding: 0,
			descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
			descriptor_count: 1,
			stage_flags: vk::ShaderStageFlags::VERTEX,
			p_immutable_samplers: ptr::null(),
		}];
		let color_normal_tex_info = vk::DescriptorSetLayoutCreateInfo {
			s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
			binding_count: color_normal_tex_dsl_bindings.len() as u32,
			p_bindings: color_normal_tex_dsl_bindings.as_ptr(),
			..Default::default()
		};
		let view_matrix_info = vk::DescriptorSetLayoutCreateInfo {
			s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
			binding_count: view_matrix_dsl_binding.len() as u32,
			p_bindings: view_matrix_dsl_binding.as_ptr(),
			..Default::default()
		};

		let descriptor_set_layouts;
		unsafe {
			descriptor_set_layouts = [
				rs.device.create_descriptor_set_layout(&color_normal_tex_info, None).unwrap(),
				rs.device.create_descriptor_set_layout(&view_matrix_info, None).unwrap(),
			];
		}

		let mv_matrices_push_constant = vk::PushConstantRange {
			stage_flags: vk::ShaderStageFlags::VERTEX,
			size: 2 * size_of::<Matrix4<f32>>() as u32,
			offset: 0,
		};

		let layout_create_info = vk::PipelineLayoutCreateInfo {
			s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
			set_layout_count: descriptor_set_layouts.len() as u32,
			p_set_layouts: descriptor_set_layouts.as_ptr(),
			push_constant_range_count: 1,
			p_push_constant_ranges: &mv_matrices_push_constant,
			..Default::default()
		};

		let pipeline_layout;
		unsafe {
			pipeline_layout = rs.device.create_pipeline_layout(&layout_create_info, None).unwrap();
		}

		let vertex_shader_module = rs.load_shader("shaders/phong_vert.spv");
		let fragment_shader_module = rs.load_shader("shaders/phong_frag.spv");

		let shader_entry_name = CString::new("main").unwrap();
		let shader_stage_create_infos = [
			vk::PipelineShaderStageCreateInfo {
				s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
				module: vertex_shader_module,
				p_name: shader_entry_name.as_ptr(),
				stage: vk::ShaderStageFlags::VERTEX,
				..Default::default()
			},
			vk::PipelineShaderStageCreateInfo {
				s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
				module: fragment_shader_module,
				p_name: shader_entry_name.as_ptr(),
				stage: vk::ShaderStageFlags::FRAGMENT,
				..Default::default()
			},
		];

		// TODO: These would probably do better to live where the Vertex struct is defined.
		let vertex_binding_description = vk::VertexInputBindingDescription {
			binding: 0,
			stride: size_of::<Vertex>() as u32,
			input_rate: vk::VertexInputRate::VERTEX,
		};

		let vertex_position_attribute_description = vk::VertexInputAttributeDescription {
			binding: 0,
			location: 0,
			format: vk::Format::R32G32B32_SFLOAT,
			offset: 0 as u32,
		};

		let vertex_normal_attribute_description = vk::VertexInputAttributeDescription {
			binding: 0,
			location: 1,
			format: vk::Format::R32G32B32_SFLOAT,
			offset: 3 * size_of::<f32>() as u32, // TODO: Make these use offset_of! macro.
		};

		let vertex_tangent_attribute_description = vk::VertexInputAttributeDescription {
			binding: 0,
			location: 2,
			format: vk::Format::R32G32B32_SFLOAT,
			offset: 6 * size_of::<f32>() as u32, // TODO: Make these use offset_of! macro.
		};

		let vertex_bitangent_attribute_description = vk::VertexInputAttributeDescription {
			binding: 0,
			location: 3,
			format: vk::Format::R32G32B32_SFLOAT,
			offset: 9 * size_of::<f32>() as u32, // TODO: Make these use offset_of! macro.
		};

		let vertex_texcoord_attribute_description = vk::VertexInputAttributeDescription {
			binding: 0,
			location: 4,
			format: vk::Format::R32G32_SFLOAT,
			offset: 12 * size_of::<f32>() as u32, // TODO: Make these use offset_of! macro.
		};

		let vertex_input_binding_descriptions = [vertex_binding_description];
		let vertex_input_attribute_descriptions = [
			vertex_position_attribute_description,
			vertex_normal_attribute_description,
			vertex_tangent_attribute_description,
			vertex_bitangent_attribute_description,
			vertex_texcoord_attribute_description,
		];
		let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
			p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
			vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
			p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
		};
		let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			primitive_restart_enable: 0,
			topology: vk::PrimitiveTopology::TRIANGLE_LIST,
		};
		let viewport = vk::Viewport {
			x: 0.0,
			y: 0.0,
			width: render_size.width as f32,
			height: render_size.height as f32,
			min_depth: 0.0,
			max_depth: 1.0,
		};
		let scissor = vk::Rect2D {
			offset: vk::Offset2D {
				x: 0,
				y: 0,
			},
			extent: vk::Extent2D {
				width: render_size.width,
				height: render_size.height,
			},
		};
		let viewport_state_info = vk::PipelineViewportStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
			scissor_count: 1,
			p_scissors: &scissor,
			viewport_count: 1,
			p_viewports: &viewport,
			..Default::default()
		};
		let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
			cull_mode: vk::CullModeFlags::BACK,
			front_face: vk::FrontFace::COUNTER_CLOCKWISE,
			line_width: 1.0,
			polygon_mode: vk::PolygonMode::FILL,
			..Default::default()
		};
		let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
			rasterization_samples: vk::SampleCountFlags::TYPE_1,
			..Default::default()
		};
		let noop_stencil_state = vk::StencilOpState {
			fail_op: vk::StencilOp::KEEP,
			pass_op: vk::StencilOp::KEEP,
			depth_fail_op: vk::StencilOp::KEEP,
			compare_op: vk::CompareOp::ALWAYS,
			..Default::default()
		};
		let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
			depth_test_enable: 1,
			depth_write_enable: 1,
			depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
			front: noop_stencil_state.clone(),
			back: noop_stencil_state.clone(),
			max_depth_bounds: 1.0,
			min_depth_bounds: 0.0,
			..Default::default()
		};
		let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
			blend_enable: 0,
			color_write_mask: vk::ColorComponentFlags::all(),
			..Default::default()
		}];
		let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
			attachment_count: color_blend_attachment_states.len() as u32,
			p_attachments: color_blend_attachment_states.as_ptr(),
			..Default::default()
		};
		let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
		let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
			dynamic_state_count: dynamic_state.len() as u32,
			p_dynamic_states: dynamic_state.as_ptr(),
			..Default::default()
		};
		let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
			s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
			stage_count: shader_stage_create_infos.len() as u32,
			p_stages: shader_stage_create_infos.as_ptr(),
			p_vertex_input_state: &vertex_input_state_info,
			p_input_assembly_state: &vertex_input_assembly_state_info,
			p_viewport_state: &viewport_state_info,
			p_rasterization_state: &rasterization_info,
			p_multisample_state: &multisample_state_info,
			p_depth_stencil_state: &depth_state_info,
			p_color_blend_state: &color_blend_state,
			p_dynamic_state: &dynamic_state_info,
			layout: pipeline_layout,
			render_pass: renderpass,
			..Default::default()
		};
		let graphics_pipelines;
		unsafe {
			graphics_pipelines = rs
				.device
				.create_graphics_pipelines(vk::PipelineCache::null(), &[graphic_pipeline_info], None)
				.expect("Unable to create graphics pipeline");

			// Graphics pipeline created, we no longer need the shader modules
			rs.device.destroy_shader_module(fragment_shader_module, None);
			rs.device.destroy_shader_module(vertex_shader_module, None);
		}

		(descriptor_pool, descriptor_set_layouts.to_vec(), pipeline_layout, viewport, scissor, graphics_pipelines[0])
	}

	/// Creates framebuffers for the presentable images, one per image.
	fn create_framebuffer(
		rs: &RenderState, render_size: vk::Extent3D, color_view: vk::ImageView, depth_view: vk::ImageView,
		renderpass: vk::RenderPass,
	) -> vk::Framebuffer
	{
		let framebuffer_attachments = [color_view, depth_view];
		let frame_buffer_create_info = vk::FramebufferCreateInfo {
			s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
			render_pass: renderpass,
			attachment_count: framebuffer_attachments.len() as u32,
			p_attachments: framebuffer_attachments.as_ptr(),
			width: render_size.width,
			height: render_size.height,
			layers: 1,
			..Default::default()
		};
		let framebuffer;
		unsafe {
			framebuffer = rs.device.create_framebuffer(&frame_buffer_create_info, None).unwrap();
		}
		framebuffer
	}

	/// Creates commandbuffer.
	fn create_commandbuffer(rs: &RenderState) -> vk::CommandBuffer
	{
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
			s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
			p_next: ptr::null(),
			command_buffer_count: 1,
			command_pool: rs.commandpool,
			level: vk::CommandBufferLevel::PRIMARY,
		};
		let commandbuffers;
		unsafe {
			commandbuffers = rs.device.allocate_command_buffers(&command_buffer_allocate_info).unwrap();
		}

		commandbuffers[0]
	}

	/// Initializes the MainPass based on a RenderState
	///
	/// This will set up the renderpass, etc.
	pub fn init(rs: &RenderState, cfg: &Config) -> MainPass
	{
		let render_format = vk::Format::R8G8B8A8_UNORM;
		let render_size = vk::Extent3D {
			width: cfg.render_width,
			height: cfg.render_height,
			depth: 1,
		};

		// Create image to render to.
		let render_image = rs.create_texture(
			render_size,
			vk::ImageType::TYPE_2D,
			vk::ImageViewType::TYPE_2D,
			render_format,
			vk::ImageAspectFlags::COLOR,
			vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
			vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
			vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
			vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
			None,
		);
		let depth_image = rs.create_texture(
			render_size,
			vk::ImageType::TYPE_2D,
			vk::ImageViewType::TYPE_2D,
			vk::Format::D32_SFLOAT,
			vk::ImageAspectFlags::DEPTH,
			vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
			vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
			vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
			vk::PipelineStageFlags::ALL_GRAPHICS,
			None,
		);

		let renderpass = MainPass::create_renderpass(rs, render_format);
		let (descriptor_pool, descriptor_set_layouts, pipeline_layout, viewport, scissor, pipeline) =
			MainPass::create_pipeline(rs, render_size, renderpass);
		let framebuffer =
			MainPass::create_framebuffer(rs, render_size, render_image.view, depth_image.view, renderpass);
		let commandbuffer = MainPass::create_commandbuffer(rs);

		let (vmat_buf, vmat_mem) = rs.create_buffer(
			vk::BufferUsageFlags::UNIFORM_BUFFER,
			vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
			size_of::<Matrix4<f32>>() as u64,
		);
		let desc_alloc_info = vk::DescriptorSetAllocateInfo {
			s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
			p_next: ptr::null(),
			descriptor_pool: descriptor_pool,
			descriptor_set_count: 1,
			p_set_layouts: &descriptor_set_layouts[1],
		};
		let view_matrix_ds;
		unsafe {
			view_matrix_ds = rs.device.allocate_descriptor_sets(&desc_alloc_info).unwrap();
		}

		MainPass {
			renderpass: renderpass,
			descriptor_pool: descriptor_pool,
			descriptor_set_layouts: descriptor_set_layouts,
			pipeline_layout: pipeline_layout,
			viewport: viewport,
			scissor: scissor,
			pipeline: pipeline,
			framebuffer: framebuffer,
			commandbuffer: commandbuffer,

			render_image: render_image,
			depth_image: depth_image,

			view_matrix_ub: vmat_buf,
			view_matrix_ub_mem: vmat_mem,
			view_matrix_ds: view_matrix_ds,

			// Keep a pointer to the device for cleanup
			device: Rc::clone(&rs.device),
		}
	}
	/// Begins the main render pass
	///
	/// Returns a command buffer to be used in rendering.
	pub fn begin_frame(&mut self, rs: &RenderState) -> vk::CommandBuffer
	{
		// Begin commandbuffer
		let cmd_buf_begin_info = vk::CommandBufferBeginInfo {
			s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
			flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
			..Default::default()
		};
		let cmd_buf = self.commandbuffer;
		unsafe {
			rs.device.begin_command_buffer(cmd_buf, &cmd_buf_begin_info).expect("Begin commandbuffer");
		}

		// Transition the mainpass output to a renderable image
		rs.transition_texture(
			&mut self.render_image,
			vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
			vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
			vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
			Some(cmd_buf),
		);

		// Begin renderpass
		let clear_values = [
			vk::ClearValue {
				color: vk::ClearColorValue {
					float32: [0.0, 1.0, 0.0, 1.0],
				},
			},
			vk::ClearValue {
				depth_stencil: vk::ClearDepthStencilValue {
					depth: 1.0,
					stencil: 0,
				},
			},
		];

		let render_pass_begin_info = vk::RenderPassBeginInfo {
			s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
			p_next: ptr::null(),
			render_pass: self.renderpass,
			framebuffer: self.framebuffer,
			render_area: self.scissor,
			clear_value_count: clear_values.len() as u32,
			p_clear_values: clear_values.as_ptr(),
		};

		let view_matrix_ub_descriptor = vk::DescriptorBufferInfo {
			buffer: self.view_matrix_ub,
			offset: 0,
			range: size_of::<Matrix4<f32>>() as u64,
		};
		let write_desc_sets = [vk::WriteDescriptorSet {
			s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
			dst_set: self.view_matrix_ds[0],
			dst_binding: 0,
			dst_array_element: 0,
			descriptor_count: 1,
			descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
			p_buffer_info: &view_matrix_ub_descriptor,
			..Default::default()
		}];

		unsafe {
			// Update the view matrix descriptor set
			rs.device.update_descriptor_sets(&write_desc_sets, &[]);

			// Start the render pass
			rs.device.cmd_begin_render_pass(cmd_buf, &render_pass_begin_info, vk::SubpassContents::INLINE);

			rs.device.cmd_bind_descriptor_sets(
				cmd_buf,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				1,
				&self.view_matrix_ds[..],
				&[],
			);

			// Bind pipeline
			rs.device.cmd_bind_pipeline(cmd_buf, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

			rs.device.cmd_set_viewport(cmd_buf, 0, &[self.viewport]);
			rs.device.cmd_set_scissor(cmd_buf, 0, &[self.scissor]);
		}

		cmd_buf
	}

	/// Ends the main render frame
	pub fn end_frame(&mut self, rs: &RenderState)
	{
		let cmd_buf = self.commandbuffer;

		unsafe {
			// End render pass and command buffer
			rs.device.cmd_end_render_pass(cmd_buf);
			rs.device.end_command_buffer(cmd_buf).expect("End commandbuffer");
		}

		// Send the work off to the GPU
		let submit_info = vk::SubmitInfo {
			s_type: vk::StructureType::SUBMIT_INFO,
			command_buffer_count: 1,
			p_command_buffers: &cmd_buf,
			..Default::default()
		};
		unsafe {
			rs.device.queue_submit(rs.graphics_queue, &[submit_info], vk::Fence::null()).expect("queue submit failed.");
		}
	}
}

impl Drop for MainPass
{
	fn drop(&mut self)
	{
		// We cannot have the last reference to device at this point
		debug_assert!(1 < Rc::strong_count(&self.device));

		unsafe {
			// Always wait for device idle
			self.device.device_wait_idle().unwrap();

			self.device.destroy_buffer(self.view_matrix_ub, None);
			self.device.free_memory(self.view_matrix_ub_mem, None);

			self.device.destroy_sampler(self.depth_image.sampler, None);
			self.device.destroy_image_view(self.depth_image.view, None);
			self.device.destroy_image(self.depth_image.image, None);
			self.device.free_memory(self.depth_image.memory, None);

			self.device.destroy_sampler(self.render_image.sampler, None);
			self.device.destroy_image_view(self.render_image.view, None);
			self.device.destroy_image(self.render_image.image, None);
			self.device.free_memory(self.render_image.memory, None);

			self.device.destroy_framebuffer(self.framebuffer, None);

			self.device.destroy_pipeline(self.pipeline, None);
			self.device.destroy_pipeline_layout(self.pipeline_layout, None);

			for &dset_layout in self.descriptor_set_layouts.iter()
			{
				self.device.destroy_descriptor_set_layout(dset_layout, None);
			}

			self.device.destroy_descriptor_pool(self.descriptor_pool, None);

			self.device.destroy_render_pass(self.renderpass, None);
		}
	}
}
