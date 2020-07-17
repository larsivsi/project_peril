use ash::extensions::khr::{Surface, Swapchain, XlibSurface};
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use ash::Device;
use std;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;
use winit;

use crate::renderer::{RenderState, Texture};

pub struct PresentPass
{
	// Surface
	surface_loader: Surface,
	surface: vk::SurfaceKHR,
	surface_format: vk::SurfaceFormatKHR,

	// Semaphores
	image_available_sem: vk::Semaphore,
	rendering_finished_sem: vk::Semaphore,

	swapchain_loader: Swapchain,

	// Swapchain
	swapchain: vk::SwapchainKHR,
	// presentable images for the screen
	present_image_views: Vec<vk::ImageView>,
	renderpass: vk::RenderPass,
	descriptor_pool: vk::DescriptorPool,
	descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
	descriptor_sets: Vec<vk::DescriptorSet>,
	pipeline_layout: vk::PipelineLayout,
	viewport: vk::Viewport,
	scissor: vk::Rect2D,
	pipeline: vk::Pipeline,
	// one framebuffer/commandbuffer per image
	framebuffers: Vec<vk::Framebuffer>,
	commandbuffers: Vec<vk::CommandBuffer>,

	// The current idx
	current_present_idx: usize,

	// Keep a pointer to the device for cleanup
	device: Rc<Device>,
}

impl PresentPass
{
	/// Creates an X11 surface.
	fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
		entry: &E, instance: &I, window: &winit::Window,
	) -> Result<vk::SurfaceKHR, vk::Result>
	{
		use winit::os::unix::WindowExt;
		let x11_display = window.get_xlib_display().unwrap();
		let x11_window = window.get_xlib_window().unwrap();
		let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
			s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
			p_next: ptr::null(),
			flags: Default::default(),
			window: x11_window as vk::Window,
			dpy: x11_display as *mut vk::Display,
		};
		let xlib_surface_loader = XlibSurface::new(entry, instance);
		let result;
		unsafe {
			result = xlib_surface_loader.create_xlib_surface(&x11_create_info, None);
		}
		result
	}

	/// Creates a vk::Swapchain and a vk::Rect2D for the current RenderState and surface.
	///
	/// Swapchain is used to queue and present stuff to the screen.
	fn create_swapchain(
		rs: &RenderState, surface_loader: &Surface, surface: &vk::SurfaceKHR, surface_format: &vk::SurfaceFormatKHR,
		old_swapchain: vk::SwapchainKHR, swapchain_loader: &Swapchain,
	) -> (vk::SwapchainKHR, vk::Rect2D)
	{
		let surface_capabilities;
		unsafe {
			surface_capabilities =
				surface_loader.get_physical_device_surface_capabilities(rs.pdevice, *surface).unwrap();
		}

		// TODO Find out why our surface wants triple buffering. Such latency, much lag.
		let mut desired_image_count = 3;
		debug_assert!(desired_image_count >= surface_capabilities.min_image_count);
		if surface_capabilities.max_image_count > 0 && desired_image_count > surface_capabilities.max_image_count
		{
			desired_image_count = surface_capabilities.max_image_count;
		}

		let pre_transform =
			if surface_capabilities.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
			{
				vk::SurfaceTransformFlagsKHR::IDENTITY
			}
			else
			{
				surface_capabilities.current_transform
			};

		let present_modes;
		unsafe {
			present_modes = surface_loader.get_physical_device_surface_present_modes(rs.pdevice, *surface).unwrap();
		}
		// Use FIFO presentmode to block on acquire_next_image, thus enabling vsync.
		let present_mode = present_modes.iter().cloned().find(|&mode| mode == vk::PresentModeKHR::FIFO).unwrap();
		let swapchain_create_info = vk::SwapchainCreateInfoKHR {
			s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
			p_next: ptr::null(),
			flags: Default::default(),
			surface: *surface,
			min_image_count: desired_image_count,
			image_color_space: surface_format.color_space,
			image_format: surface_format.format,
			image_extent: surface_capabilities.current_extent.clone(),
			image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
			image_sharing_mode: vk::SharingMode::EXCLUSIVE,
			pre_transform: pre_transform,
			composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
			present_mode: present_mode,
			clipped: 1,
			old_swapchain: old_swapchain,
			image_array_layers: 1,
			p_queue_family_indices: ptr::null(),
			queue_family_index_count: 0,
		};
		let swapchain;
		unsafe {
			swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None).unwrap();
		}

		(
			swapchain,
			vk::Rect2D {
				offset: vk::Offset2D {
					x: 0,
					y: 0,
				},
				extent: surface_capabilities.current_extent.clone(),
			},
		)
	}

	/// Creates a Vec of vk::ImageViews for the presentable images in the swapchain.
	///
	/// This will create two imageviews for double-buffering, three imageviews for
	/// tripple-buffering etc.
	fn create_imageviews(
		rs: &RenderState, surface_format: &vk::SurfaceFormatKHR, swapchain_loader: &Swapchain,
		swapchain: vk::SwapchainKHR,
	) -> Vec<vk::ImageView>
	{
		let present_images;
		unsafe {
			present_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
		}
		let present_image_views: Vec<vk::ImageView> = present_images
			.iter()
			.map(|&image| {
				let create_view_info = vk::ImageViewCreateInfo {
					s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
					p_next: ptr::null(),
					flags: Default::default(),
					view_type: vk::ImageViewType::TYPE_2D,
					format: surface_format.format,
					components: vk::ComponentMapping {
						r: vk::ComponentSwizzle::R,
						g: vk::ComponentSwizzle::G,
						b: vk::ComponentSwizzle::B,
						a: vk::ComponentSwizzle::A,
					},
					subresource_range: vk::ImageSubresourceRange {
						aspect_mask: vk::ImageAspectFlags::COLOR,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					},
					image: image,
				};
				let result;
				unsafe { result = rs.device.create_image_view(&create_view_info, None).unwrap() }
				result
			})
			.collect();

		present_image_views
	}

	/// Creates a presentable renderpass.
	///
	/// Produces a color-only renderpass, perfect for direct drawing.
	fn create_renderpass(rs: &RenderState, surface_format: &vk::SurfaceFormatKHR) -> vk::RenderPass
	{
		// One attachment, color only. Will produce the presentable image.
		let renderpass_attachments = [vk::AttachmentDescription {
			format: surface_format.format,
			flags: vk::AttachmentDescriptionFlags::empty(),
			samples: vk::SampleCountFlags::TYPE_1,
			load_op: vk::AttachmentLoadOp::DONT_CARE,
			store_op: vk::AttachmentStoreOp::STORE,
			stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
			stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
			initial_layout: vk::ImageLayout::UNDEFINED,
			final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
		}];
		let color_attachment_ref = vk::AttachmentReference {
			attachment: 0,
			layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
		};
		let subpass = vk::SubpassDescription {
			color_attachment_count: 1,
			p_color_attachments: &color_attachment_ref,
			p_depth_stencil_attachment: ptr::null(),
			flags: Default::default(),
			pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
			input_attachment_count: 0,
			p_input_attachments: ptr::null(),
			p_resolve_attachments: ptr::null(),
			preserve_attachment_count: 0,
			p_preserve_attachments: ptr::null(),
		};
		let renderpass_create_info = vk::RenderPassCreateInfo {
			s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			attachment_count: renderpass_attachments.len() as u32,
			p_attachments: renderpass_attachments.as_ptr(),
			subpass_count: 1,
			p_subpasses: &subpass,
			dependency_count: 0,
			p_dependencies: ptr::null(),
		};
		let renderpass;
		unsafe {
			renderpass = rs.device.create_render_pass(&renderpass_create_info, None).unwrap();
		}

		renderpass
	}

	/// Creates a pipeline for the given presentable renderpass.
	///
	/// Very straigt forward pipeline: Loads some hard-coded shaders that will draw a triangle.
	fn create_pipeline(
		rs: &RenderState, surface_size: vk::Rect2D, renderpass: vk::RenderPass,
	) -> (
		vk::DescriptorPool,
		Vec<vk::DescriptorSetLayout>,
		Vec<vk::DescriptorSet>,
		vk::PipelineLayout,
		vk::Viewport,
		vk::Rect2D,
		vk::Pipeline,
	)
	{
		// Descriptors
		let descriptor_sizes = [vk::DescriptorPoolSize {
			ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
			descriptor_count: 1,
		}];
		let descriptor_pool_info = vk::DescriptorPoolCreateInfo {
			s_type: vk::StructureType::DESCRIPTOR_POOL_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			pool_size_count: descriptor_sizes.len() as u32,
			p_pool_sizes: descriptor_sizes.as_ptr(),
			max_sets: 1,
		};
		let descriptor_pool;
		unsafe {
			descriptor_pool = rs.device.create_descriptor_pool(&descriptor_pool_info, None).unwrap();
		}
		let desc_layout_bindings = [vk::DescriptorSetLayoutBinding {
			binding: 0,
			descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
			descriptor_count: 1,
			stage_flags: vk::ShaderStageFlags::FRAGMENT,
			p_immutable_samplers: ptr::null(),
		}];
		let descriptor_info = vk::DescriptorSetLayoutCreateInfo {
			s_type: vk::StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			binding_count: desc_layout_bindings.len() as u32,
			p_bindings: desc_layout_bindings.as_ptr(),
		};
		let descriptor_set_layouts;
		unsafe {
			descriptor_set_layouts = [rs.device.create_descriptor_set_layout(&descriptor_info, None).unwrap()];
		}
		let desc_alloc_info = vk::DescriptorSetAllocateInfo {
			s_type: vk::StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
			p_next: ptr::null(),
			descriptor_pool: descriptor_pool,
			descriptor_set_count: descriptor_set_layouts.len() as u32,
			p_set_layouts: descriptor_set_layouts.as_ptr(),
		};
		let descriptor_sets;
		unsafe {
			descriptor_sets = rs.device.allocate_descriptor_sets(&desc_alloc_info).unwrap();
		}
		let layout_create_info = vk::PipelineLayoutCreateInfo {
			s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			set_layout_count: descriptor_set_layouts.len() as u32,
			p_set_layouts: descriptor_set_layouts.as_ptr(),
			push_constant_range_count: 0,
			p_push_constant_ranges: ptr::null(),
		};

		let pipeline_layout;
		unsafe {
			pipeline_layout = rs.device.create_pipeline_layout(&layout_create_info, None).unwrap();
		}

		let vertex_shader_module = rs.load_shader("shaders/final_pass_vert.spv");
		let fragment_shader_module = rs.load_shader("shaders/final_pass_frag.spv");

		let shader_entry_name = CString::new("main").unwrap();
		let shader_stage_create_infos = [
			vk::PipelineShaderStageCreateInfo {
				s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
				p_next: ptr::null(),
				flags: Default::default(),
				module: vertex_shader_module,
				p_name: shader_entry_name.as_ptr(),
				p_specialization_info: ptr::null(),
				stage: vk::ShaderStageFlags::VERTEX,
			},
			vk::PipelineShaderStageCreateInfo {
				s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
				p_next: ptr::null(),
				flags: Default::default(),
				module: fragment_shader_module,
				p_name: shader_entry_name.as_ptr(),
				p_specialization_info: ptr::null(),
				stage: vk::ShaderStageFlags::FRAGMENT,
			},
		];
		let vertex_input_binding_descriptions = [];
		let vertex_input_attribute_descriptions = [];
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
			x: surface_size.offset.x as f32,
			y: surface_size.offset.y as f32,
			width: surface_size.extent.width as f32,
			height: surface_size.extent.height as f32,
			min_depth: 0.0,
			max_depth: 1.0,
		};
		let scissor = surface_size.clone();
		let viewport_state_info = vk::PipelineViewportStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			scissor_count: 1,
			p_scissors: &scissor,
			viewport_count: 1,
			p_viewports: &viewport,
		};
		let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			cull_mode: vk::CullModeFlags::BACK,
			depth_bias_clamp: 0.0,
			depth_bias_constant_factor: 0.0,
			depth_bias_enable: 0,
			depth_bias_slope_factor: 0.0,
			depth_clamp_enable: 0,
			front_face: vk::FrontFace::COUNTER_CLOCKWISE,
			line_width: 1.0,
			polygon_mode: vk::PolygonMode::FILL,
			rasterizer_discard_enable: 0,
		};
		let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			rasterization_samples: vk::SampleCountFlags::TYPE_1,
			sample_shading_enable: 0,
			min_sample_shading: 0.0,
			p_sample_mask: ptr::null(),
			alpha_to_one_enable: 0,
			alpha_to_coverage_enable: 0,
		};
		let noop_stencil_state = vk::StencilOpState {
			fail_op: vk::StencilOp::KEEP,
			pass_op: vk::StencilOp::KEEP,
			depth_fail_op: vk::StencilOp::KEEP,
			compare_op: vk::CompareOp::ALWAYS,
			compare_mask: 0,
			write_mask: 0,
			reference: 0,
		};
		let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			depth_test_enable: 1,
			depth_write_enable: 1,
			depth_compare_op: vk::CompareOp::LESS_OR_EQUAL,
			depth_bounds_test_enable: 0,
			stencil_test_enable: 0,
			front: noop_stencil_state.clone(),
			back: noop_stencil_state.clone(),
			max_depth_bounds: 1.0,
			min_depth_bounds: 0.0,
		};
		let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
			blend_enable: 0,
			src_color_blend_factor: vk::BlendFactor::SRC_COLOR,
			dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_DST_COLOR,
			color_blend_op: vk::BlendOp::ADD,
			src_alpha_blend_factor: vk::BlendFactor::ZERO,
			dst_alpha_blend_factor: vk::BlendFactor::ZERO,
			alpha_blend_op: vk::BlendOp::ADD,
			color_write_mask: vk::ColorComponentFlags::all(),
		}];
		let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			logic_op_enable: 0,
			logic_op: vk::LogicOp::CLEAR,
			attachment_count: color_blend_attachment_states.len() as u32,
			p_attachments: color_blend_attachment_states.as_ptr(),
			blend_constants: [0.0, 0.0, 0.0, 0.0],
		};
		let dynamic_state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
		let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
			s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
			dynamic_state_count: dynamic_state.len() as u32,
			p_dynamic_states: dynamic_state.as_ptr(),
		};
		let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
			s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::PipelineCreateFlags::empty(),
			stage_count: shader_stage_create_infos.len() as u32,
			p_stages: shader_stage_create_infos.as_ptr(),
			p_vertex_input_state: &vertex_input_state_info,
			p_input_assembly_state: &vertex_input_assembly_state_info,
			p_tessellation_state: ptr::null(),
			p_viewport_state: &viewport_state_info,
			p_rasterization_state: &rasterization_info,
			p_multisample_state: &multisample_state_info,
			p_depth_stencil_state: &depth_state_info,
			p_color_blend_state: &color_blend_state,
			p_dynamic_state: &dynamic_state_info,
			layout: pipeline_layout,
			render_pass: renderpass,
			subpass: 0,
			base_pipeline_handle: vk::Pipeline::null(),
			base_pipeline_index: 0,
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

		(
			descriptor_pool,
			descriptor_set_layouts.to_vec(),
			descriptor_sets,
			pipeline_layout,
			viewport,
			scissor,
			graphics_pipelines[0],
		)
	}

	/// Creates framebuffers for the presentable images, one per image.
	fn create_framebuffers(
		rs: &RenderState, surface_size: vk::Rect2D, present_image_views: &Vec<vk::ImageView>,
		renderpass: vk::RenderPass,
	) -> Vec<vk::Framebuffer>
	{
		let framebuffers: Vec<vk::Framebuffer> = present_image_views
			.iter()
			.map(|&present_image_view| {
				let framebuffer_attachments = [present_image_view];
				let frame_buffer_create_info = vk::FramebufferCreateInfo {
					s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
					p_next: ptr::null(),
					flags: Default::default(),
					render_pass: renderpass,
					attachment_count: framebuffer_attachments.len() as u32,
					p_attachments: framebuffer_attachments.as_ptr(),
					width: surface_size.extent.width,
					height: surface_size.extent.height,
					layers: 1,
				};
				let framebuffer;
				unsafe {
					framebuffer = rs.device.create_framebuffer(&frame_buffer_create_info, None).unwrap();
				}
				framebuffer
			})
			.collect();

		framebuffers
	}

	/// Creates commandbuffers for the presentable images, one per image.
	fn create_commandbuffers(rs: &RenderState, framebuffers: &Vec<vk::Framebuffer>) -> Vec<vk::CommandBuffer>
	{
		let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
			s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
			p_next: ptr::null(),
			command_buffer_count: framebuffers.len() as u32,
			command_pool: rs.commandpool,
			level: vk::CommandBufferLevel::PRIMARY,
		};
		let command_buffers;
		unsafe {
			command_buffers = rs.device.allocate_command_buffers(&command_buffer_allocate_info).unwrap();
		}

		command_buffers
	}

	/// Initializes the PresentPass based on a RenderState
	///
	/// This will set up the swapchain, renderpass, etc.
	pub fn init(rs: &RenderState) -> PresentPass
	{
		// Surface
		let surface_loader = Surface::new(&rs.entry, &rs.instance);
		let surface = PresentPass::create_surface(&rs.entry, &rs.instance, &rs.window).unwrap();
		let surface_formats;
		unsafe {
			assert!(surface_loader
				.get_physical_device_surface_support(rs.pdevice, rs.queue_family_index, surface)
				.unwrap());
			surface_formats = surface_loader.get_physical_device_surface_formats(rs.pdevice, surface).unwrap();
		}
		let surface_format = surface_formats
			.iter()
			.map(|sfmt| match sfmt.format
			{
				vk::Format::UNDEFINED => vk::SurfaceFormatKHR {
					format: vk::Format::B8G8R8_UNORM,
					color_space: sfmt.color_space,
				},
				_ => sfmt.clone(),
			})
			.nth(0)
			.expect("Unable to find suitable surface format.");

		let sem_create_info = vk::SemaphoreCreateInfo {
			s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
			p_next: ptr::null(),
			flags: Default::default(),
		};
		let image_available_sem;
		let rendering_finished_sem;
		unsafe {
			image_available_sem = rs.device.create_semaphore(&sem_create_info, None).unwrap();
			rendering_finished_sem = rs.device.create_semaphore(&sem_create_info, None).unwrap();
		}

		let swapchain_loader = Swapchain::new(&rs.instance, rs.device.as_ref());

		let (swapchain, surface_size) = PresentPass::create_swapchain(
			rs,
			&surface_loader,
			&surface,
			&surface_format,
			vk::SwapchainKHR::null(),
			&swapchain_loader,
		);
		let present_image_views = PresentPass::create_imageviews(rs, &surface_format, &swapchain_loader, swapchain);
		let renderpass = PresentPass::create_renderpass(rs, &surface_format);
		let (descriptor_pool, descriptor_set_layouts, descriptor_sets, pipeline_layout, viewport, scissor, pipeline) =
			PresentPass::create_pipeline(rs, surface_size, renderpass);
		let framebuffers = PresentPass::create_framebuffers(rs, surface_size, &present_image_views, renderpass);
		let command_buffers = PresentPass::create_commandbuffers(rs, &framebuffers);

		PresentPass {
			// Surface
			surface_loader: surface_loader,
			surface: surface,
			surface_format: surface_format,

			// Semaphores
			image_available_sem: image_available_sem,
			rendering_finished_sem: rendering_finished_sem,

			swapchain_loader: swapchain_loader,

			// Swapchain
			swapchain: swapchain,
			present_image_views: present_image_views,
			renderpass: renderpass,
			descriptor_pool: descriptor_pool,
			descriptor_set_layouts: descriptor_set_layouts,
			descriptor_sets: descriptor_sets,
			pipeline_layout: pipeline_layout,
			viewport: viewport,
			scissor: scissor,
			pipeline: pipeline,
			// one framebuffer/commandbuffer per image
			framebuffers: framebuffers,
			commandbuffers: command_buffers,

			// The current idx
			current_present_idx: std::usize::MAX,

			// Keep a pointer to the device for cleanup
			device: Rc::clone(&rs.device),
		}
	}

	/// Releases all resources for the currently bound swapchain.
	///
	/// The user is responsible for not calling this function without a swapchain.
	fn cleanup_swapchain(&mut self)
	{
		unsafe {
			// Always wait for device idle
			self.device.device_wait_idle().unwrap();

			for &framebuffer in self.framebuffers.iter()
			{
				self.device.destroy_framebuffer(framebuffer, None);
			}

			self.device.destroy_pipeline(self.pipeline, None);
			self.device.destroy_pipeline_layout(self.pipeline_layout, None);

			for &dset_layout in self.descriptor_set_layouts.iter()
			{
				self.device.destroy_descriptor_set_layout(dset_layout, None);
			}

			self.device.destroy_descriptor_pool(self.descriptor_pool, None);

			self.device.destroy_render_pass(self.renderpass, None);

			for &image_view in self.present_image_views.iter()
			{
				self.device.destroy_image_view(image_view, None);
			}

			self.swapchain_loader.destroy_swapchain(self.swapchain, None);
		}
	}

	/// Releases the old and creates a new swapchain.
	///
	/// This function should be called when the presentable surface is resized, etc.
	fn recreate_swapchain(&mut self, rs: &RenderState)
	{
		self.cleanup_swapchain();

		let (swapchain, surface_size) = PresentPass::create_swapchain(
			rs,
			&self.surface_loader,
			&self.surface,
			&self.surface_format,
			vk::SwapchainKHR::null(),
			&self.swapchain_loader,
		);
		self.swapchain = swapchain;
		let present_image_views =
			PresentPass::create_imageviews(rs, &self.surface_format, &self.swapchain_loader, swapchain);
		self.present_image_views = present_image_views;
		let renderpass = PresentPass::create_renderpass(rs, &self.surface_format);
		self.renderpass = renderpass;
		let (descriptor_pool, descriptor_set_layouts, descriptor_sets, pipeline_layout, viewport, scissor, pipeline) =
			PresentPass::create_pipeline(rs, surface_size, renderpass);
		self.descriptor_pool = descriptor_pool;
		self.descriptor_set_layouts = descriptor_set_layouts;
		self.descriptor_sets = descriptor_sets;
		self.pipeline_layout = pipeline_layout;
		self.viewport = viewport;
		self.scissor = scissor;
		self.pipeline = pipeline;
		let framebuffers = PresentPass::create_framebuffers(rs, surface_size, &self.present_image_views, renderpass);
		self.framebuffers = framebuffers;
		let command_buffers = PresentPass::create_commandbuffers(rs, &self.framebuffers);
		self.commandbuffers = command_buffers;
	}

	/// Starts a frame for the current swapchain. The returned commandbuffer should be used for
	/// rendering.
	///
	/// On error (for example when the swapchain needs to be recreated), this function returns
	/// None, meaning that the current frame should be skipped.
	fn begin_frame(&mut self, rs: &RenderState, image: &mut Texture) -> Option<vk::CommandBuffer>
	{
		let result;
		unsafe {
			result = self.swapchain_loader.acquire_next_image(
				self.swapchain,
				std::u64::MAX,
				self.image_available_sem,
				vk::Fence::null(),
			);
		}

		match result
		{
			Ok((idx, suboptimal)) =>
			{
				debug_assert!(!suboptimal);
				self.current_present_idx = idx as usize;
			}
			Err(vkres) =>
			{
				if vkres == vk::Result::ERROR_OUT_OF_DATE_KHR
				{
					self.recreate_swapchain(rs);
					return None;
				}
			}
		}

		// Begin commandbuffer
		let cmd_buf_begin_info = vk::CommandBufferBeginInfo {
			s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
			p_next: ptr::null(),
			p_inheritance_info: ptr::null(),
			flags: vk::CommandBufferUsageFlags::SIMULTANEOUS_USE,
		};
		let cmd_buf = self.commandbuffers[self.current_present_idx];
		unsafe {
			rs.device.begin_command_buffer(cmd_buf, &cmd_buf_begin_info).expect("Begin commandbuffer");
		}

		// Transition the mainpass output to a samplable image
		rs.transition_texture(
			image,
			vk::AccessFlags::SHADER_READ,
			vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
			vk::PipelineStageFlags::FRAGMENT_SHADER,
			Some(cmd_buf),
		);

		// Begin renderpass
		let render_pass_begin_info = vk::RenderPassBeginInfo {
			s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
			p_next: ptr::null(),
			render_pass: self.renderpass,
			framebuffer: self.framebuffers[self.current_present_idx],
			render_area: self.scissor,
			clear_value_count: 0,
			p_clear_values: ptr::null(),
		};
		unsafe {
			// Start the render pass
			rs.device.cmd_begin_render_pass(cmd_buf, &render_pass_begin_info, vk::SubpassContents::INLINE);

			// Bind pipeline
			rs.device.cmd_bind_pipeline(cmd_buf, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

			rs.device.cmd_set_viewport(cmd_buf, 0, &[self.viewport]);
			rs.device.cmd_set_scissor(cmd_buf, 0, &[self.scissor]);
		}

		Some(cmd_buf)
	}

	/// Ends the current frame and presents it.
	///
	/// begin_frame() must have been called before this function.
	fn end_frame_and_present(&mut self, rs: &RenderState)
	{
		debug_assert!(self.current_present_idx < std::usize::MAX);

		let cmd_buf = self.commandbuffers[self.current_present_idx];
		unsafe {
			// End render pass and command buffer
			rs.device.cmd_end_render_pass(cmd_buf);
			rs.device.end_command_buffer(cmd_buf).expect("End commandbuffer");
		}

		// Send the work off to the GPU
		let fence_create_info = vk::FenceCreateInfo {
			s_type: vk::StructureType::FENCE_CREATE_INFO,
			p_next: ptr::null(),
			flags: vk::FenceCreateFlags::empty(),
		};
		let submit_fence;
		unsafe {
			submit_fence = rs.device.create_fence(&fence_create_info, None).expect("Create fence failed.");
		}
		let submit_info = vk::SubmitInfo {
			s_type: vk::StructureType::SUBMIT_INFO,
			p_next: ptr::null(),
			wait_semaphore_count: 1,
			p_wait_semaphores: &self.image_available_sem,
			p_wait_dst_stage_mask: &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
			command_buffer_count: 1,
			p_command_buffers: &cmd_buf,
			signal_semaphore_count: 1,
			p_signal_semaphores: &self.rendering_finished_sem,
		};
		unsafe {
			rs.device.queue_submit(rs.graphics_queue, &[submit_info], submit_fence).expect("queue submit failed.");
			rs.device.wait_for_fences(&[submit_fence], true, std::u64::MAX).expect("Wait for fence failed.");
			rs.device.destroy_fence(submit_fence, None);
		}

		let present_info = vk::PresentInfoKHR {
			s_type: vk::StructureType::PRESENT_INFO_KHR,
			p_next: ptr::null(),
			wait_semaphore_count: 1,
			p_wait_semaphores: &self.rendering_finished_sem,
			swapchain_count: 1,
			p_swapchains: &self.swapchain,
			p_image_indices: &(self.current_present_idx as u32),
			p_results: ptr::null_mut(),
		};
		unsafe {
			self.swapchain_loader.queue_present(rs.graphics_queue, &present_info).unwrap();
		}

		// Make sure we call begin_frame() before calling this function again
		self.current_present_idx = std::usize::MAX;
	}

	/// Presents the passed image to the screen.
	///
	/// If swapchain is outdated, a new one is created, but no image output is done.
	pub fn present_image(&mut self, rs: &RenderState, image: &mut Texture)
	{
		let cmd_buf;
		let res = self.begin_frame(rs, image);
		match res
		{
			Some(buf) =>
			{
				cmd_buf = buf;
			}
			None =>
			{
				// Swapchain was outdated, but now one was created.
				// Skip this frame.
				return;
			}
		}
		// Draw stuff
		let image_descriptor = vk::DescriptorImageInfo {
			image_layout: image.current_layout,
			image_view: image.view,
			sampler: image.sampler,
		};
		let write_desc_sets = [vk::WriteDescriptorSet {
			s_type: vk::StructureType::WRITE_DESCRIPTOR_SET,
			p_next: ptr::null(),
			dst_set: self.descriptor_sets[0],
			dst_binding: 0,
			dst_array_element: 0,
			descriptor_count: 1,
			descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
			p_image_info: &image_descriptor,
			p_buffer_info: ptr::null(),
			p_texel_buffer_view: ptr::null(),
		}];
		unsafe {
			// Update the descriptor set for the image to draw
			rs.device.update_descriptor_sets(&write_desc_sets, &[]);
			// ...and bind it
			rs.device.cmd_bind_descriptor_sets(
				cmd_buf,
				vk::PipelineBindPoint::GRAPHICS,
				self.pipeline_layout,
				0,
				&self.descriptor_sets[..],
				&[],
			);

			// We have a hardcoded quad shader, so just draw three vertices
			rs.device.cmd_draw(cmd_buf, 3, 1, 0, 0);
		}
		// then swapbuffers etc.
		self.end_frame_and_present(rs);
	}
}

impl Drop for PresentPass
{
	/// Drops the PresentPass. This destroys the swapchain and surface.
	fn drop(&mut self)
	{
		// We cannot have the last reference to device at this point
		debug_assert!(1 < Rc::strong_count(&self.device));

		// Already contains a device wait
		self.cleanup_swapchain();

		unsafe {
			self.device.destroy_semaphore(self.rendering_finished_sem, None);
			self.device.destroy_semaphore(self.image_available_sem, None);
			self.surface_loader.destroy_surface(self.surface, None);
		}
	}
}
