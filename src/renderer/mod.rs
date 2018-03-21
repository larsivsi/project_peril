use ash::{Device, Entry, Instance};
use ash::extensions::{DebugReport, Surface, Swapchain, XlibSurface};
use ash::util::Align;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0, V1_0};
use ash::vk;
use image;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;
use std::mem::{align_of, size_of};
use std::path::Path;
use std::ptr;
use std::rc::Rc;
use winit;
use winit::EventsLoop;
use winit::Window;

mod mainpass;
mod presentpass;

pub use self::mainpass::MainPass;
pub use self::presentpass::PresentPass;

use config::Config;

pub struct Texture
{
	pub image: vk::Image,
	pub memory: vk::DeviceMemory,
	pub view: vk::ImageView,
	pub sampler: vk::Sampler,
	current_access_mask: vk::AccessFlags,
	pub current_layout: vk::ImageLayout,
	current_stage: vk::PipelineStageFlags,
}

pub struct RenderState
{
	// Vulkan device
	entry: Entry<V1_0>,
	instance: Instance<V1_0>,
	debug_report_loader: Option<DebugReport>,
	debug_callback: Option<vk::DebugReportCallbackEXT>,
	pdevice: vk::PhysicalDevice,
	pub device: Rc<Device<V1_0>>,
	device_memory_properties: vk::PhysicalDeviceMemoryProperties,
	queue_family_index: u32,
	graphics_queue: vk::Queue,

	// Window
	pub event_loop: EventsLoop,
	window: Window,

	// Pools
	commandpool: vk::CommandPool,
}

impl RenderState
{
	/// Lists the extensions required by the application.
	fn extension_names() -> Vec<*const i8>
	{
		let mut extensions = vec![Surface::name().as_ptr(), XlibSurface::name().as_ptr()];
		if cfg!(feature = "debug_layer")
		{
			extensions.push(DebugReport::name().as_ptr());
		}
		extensions
	}

	/// Creates a Vulkan instance.
	fn create_instance(cfg: &Config, entry: &Entry<V1_0>) -> Instance<V1_0>
	{
		// Application info
		let app_name = CString::new(cfg.app_name.clone()).unwrap();
		let raw_name = app_name.as_ptr();
		let appinfo = vk::ApplicationInfo {
			s_type: vk::StructureType::ApplicationInfo,
			p_next: ptr::null(),
			p_application_name: raw_name,
			application_version: cfg.app_version,
			p_engine_name: raw_name,
			engine_version: cfg.app_version,
			api_version: vk_make_version!(1, 0, 57),
		};

		// Layers
		let mut layer_names_raw: Vec<*const i8> = Vec::new();
		let requested_layers = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
		// Only enable debug layers if requested
		if cfg!(feature = "debug_layer")
		{
			println!("Debug layers:");
			let available_layers = entry.enumerate_instance_layer_properties().unwrap();
			for layer in available_layers.iter()
			{
				let layer_name;
				unsafe {
					layer_name = CStr::from_ptr(layer.layer_name.as_ptr());
				}
				println!("Found layer {:?}", layer_name);
				for req_layer in requested_layers.iter()
				{
					if layer_name == req_layer.as_c_str()
					{
						println!("Will enable {:?}", req_layer);
						layer_names_raw.push(req_layer.as_ptr());
					}
				}
			}

			println!("Will enable {} debug layers", layer_names_raw.len());
		}

		// Instance
		let extension_names_raw = RenderState::extension_names();
		let create_info = vk::InstanceCreateInfo {
			s_type: vk::StructureType::InstanceCreateInfo,
			p_next: ptr::null(),
			flags: Default::default(),
			p_application_info: &appinfo,
			pp_enabled_layer_names: layer_names_raw.as_ptr(),
			enabled_layer_count: layer_names_raw.len() as u32,
			pp_enabled_extension_names: extension_names_raw.as_ptr(),
			enabled_extension_count: extension_names_raw.len() as u32,
		};
		let instance;
		unsafe {
			instance = entry.create_instance(&create_info, None).expect("Instance creation error");
		}

		instance
	}

	/// Debug layer callback function.
	///
	/// This function is called from the debug layer if an issue is identified.
	unsafe extern "system" fn vulkan_debug_callback(
		_: vk::DebugReportFlagsEXT, _: vk::DebugReportObjectTypeEXT, _: vk::uint64_t, _: vk::size_t, _: vk::int32_t,
		_: *const vk::c_char, p_message: *const vk::c_char, _: *mut vk::c_void,
	) -> u32
	{
		println!("{:?}", CStr::from_ptr(p_message));
		1
	}

	/// Sets up the debug report layer and callback.
	fn setup_debug_callback(entry: &Entry<V1_0>, instance: &Instance<V1_0>)
		-> (DebugReport, vk::DebugReportCallbackEXT)
	{
		let debug_info = vk::DebugReportCallbackCreateInfoEXT {
			s_type: vk::StructureType::DebugReportCallbackCreateInfoExt,
			p_next: ptr::null(),
			flags: vk::DEBUG_REPORT_ERROR_BIT_EXT | vk::DEBUG_REPORT_WARNING_BIT_EXT |
				vk::DEBUG_REPORT_PERFORMANCE_WARNING_BIT_EXT,
			pfn_callback: RenderState::vulkan_debug_callback,
			p_user_data: ptr::null_mut(),
		};
		let debug_report_loader = DebugReport::new(entry, instance).unwrap();
		let debug_callback;
		unsafe {
			debug_callback = debug_report_loader.create_debug_report_callback_ext(&debug_info, None).unwrap();
		}

		(debug_report_loader, debug_callback)
	}

	/// Selects a physical device (and queue index) for the Vulkan instance.
	fn pick_physical_device(instance: &Instance<V1_0>) -> (vk::PhysicalDevice, u32)
	{
		let pdevices = instance.enumerate_physical_devices().expect("Failed to find GPU with Vulkan support");
		let (pdevice, queue_family_index) = pdevices
			.iter()
			.map(|pdevice| {
				instance
					.get_physical_device_queue_family_properties(*pdevice)
					.iter()
					.enumerate()
					.filter_map(|(index, ref info)| {
						let supports_graphics =
                                // Any GPU that can render
                                info.queue_flags.subset(vk::QUEUE_GRAPHICS_BIT);
						match supports_graphics
						{
							true => Some((*pdevice, index)),
							_ => None,
						}
					})
					.nth(0)
			})
			.filter_map(|v| v)
			.nth(0)
			.expect("Couldn't find suitable device.");

		(pdevice, queue_family_index as u32)
	}

	/// Creates a Vulkan device (logical) based on the instance and physical device.
	fn create_logical_device(
		instance: &Instance<V1_0>, pdevice: vk::PhysicalDevice, queue_family_index: u32
	) -> Device<V1_0>
	{
		let queue_priorities = [1.0]; // One queue of priority 1.0
		let queue_info = vk::DeviceQueueCreateInfo {
			s_type: vk::StructureType::DeviceQueueCreateInfo,
			p_next: ptr::null(),
			flags: Default::default(),
			queue_family_index: queue_family_index,
			p_queue_priorities: queue_priorities.as_ptr(),
			queue_count: queue_priorities.len() as u32,
		};
		let device_extension_names_raw = [Swapchain::name().as_ptr()]; // VK_KHR_swapchain
		let features = vk::PhysicalDeviceFeatures {
			shader_clip_distance: vk::VK_TRUE,
			// Can request more stuff here later
			..Default::default()
		};
		let device_create_info = vk::DeviceCreateInfo {
			s_type: vk::StructureType::DeviceCreateInfo,
			p_next: ptr::null(),
			flags: Default::default(),
			queue_create_info_count: 1,
			p_queue_create_infos: &queue_info,
			enabled_layer_count: 0,
			pp_enabled_layer_names: ptr::null(),
			enabled_extension_count: device_extension_names_raw.len() as u32,
			pp_enabled_extension_names: device_extension_names_raw.as_ptr(),
			p_enabled_features: &features,
		};
		let device: Device<V1_0>;
		unsafe {
			device =
				instance.create_device(pdevice, &device_create_info, None).expect("Failed to create logical device");
		}

		device
	}

	/// Creates various pools required by the RenderState.
	fn create_pools(device: &Device<V1_0>, queue_family_index: u32) -> (vk::CommandPool)
	{
		let cmd_pool_create_info = vk::CommandPoolCreateInfo {
			s_type: vk::StructureType::CommandPoolCreateInfo,
			p_next: ptr::null(),
			flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
			queue_family_index: queue_family_index,
		};
		let commandpool;
		unsafe { commandpool = device.create_command_pool(&cmd_pool_create_info, None).unwrap() }

		(commandpool)
	}

	/// Initializes the RenderState based in the passed Config.
	pub fn init(cfg: &Config) -> RenderState
	{
		// Window and event handler
		let event_loop = winit::EventsLoop::new();
		let window = winit::WindowBuilder::new()
			.with_title(format!("{} {}", cfg.app_name, cfg.version_to_string()))
			.with_dimensions(cfg.window_width, cfg.window_height)
			.build(&event_loop)
			.unwrap();

		// ash entry point
		let entry: Entry<V1_0> = Entry::new().unwrap();

		// Vulkan init
		let instance = RenderState::create_instance(&cfg, &entry);
		let mut debug_report_loader = None;
		let mut debug_callback = None;
		if cfg!(feature = "debug_layer")
		{
			let (loader, callback) = RenderState::setup_debug_callback(&entry, &instance);
			debug_report_loader = Some(loader);
			debug_callback = Some(callback);
		}
		let (pdevice, queue_family_index) = RenderState::pick_physical_device(&instance);
		let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);
		let device = RenderState::create_logical_device(&instance, pdevice, queue_family_index);
		let graphics_queue;
		unsafe {
			graphics_queue = device.get_device_queue(queue_family_index, 0);
		}

		// Other stuff
		let commandpool = RenderState::create_pools(&device, queue_family_index);

		// Return the RenderState
		RenderState {
			// Vulkan device
			entry: entry,
			instance: instance,
			debug_report_loader: debug_report_loader,
			debug_callback: debug_callback,
			pdevice: pdevice,
			device: Rc::new(device),
			device_memory_properties: device_memory_properties,
			queue_family_index: queue_family_index,
			graphics_queue: graphics_queue,

			// Window
			event_loop: event_loop,
			window: window,

			// Pools
			commandpool: commandpool,
		}
	}

	/// Returns a suitable memory type for the requirements based in the physical Vulkan device.
	fn find_memory_type(&self, mem_type_bits: u32, properties: vk::MemoryPropertyFlags) -> u32
	{
		for (idx, mem_type) in self.device_memory_properties.memory_types.iter().enumerate()
		{
			if mem_type_bits & (1 << idx) != 0 && (mem_type.property_flags & properties) == properties
			{
				return idx as u32;
			}
		}
		panic!("Cannot find memory type!");
	}

	/// Begins a commandbuffer that can be used for small GPU operations.
	fn begin_single_time_commands(&self) -> vk::CommandBuffer
	{
		let cmd_buf_allocate_info = vk::CommandBufferAllocateInfo {
			s_type: vk::StructureType::CommandBufferAllocateInfo,
			p_next: ptr::null(),
			command_buffer_count: 1,
			command_pool: self.commandpool,
			level: vk::CommandBufferLevel::Primary,
		};

		let cmd_buf;
		unsafe {
			cmd_buf = self.device.allocate_command_buffers(&cmd_buf_allocate_info).unwrap()[0];
		}

		let cmd_buf_begin_info = vk::CommandBufferBeginInfo {
			s_type: vk::StructureType::CommandBufferBeginInfo,
			p_next: ptr::null(),
			p_inheritance_info: ptr::null(),
			flags: vk::COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
		};
		unsafe {
			self.device.begin_command_buffer(cmd_buf, &cmd_buf_begin_info).expect("Begin commandbuffer");
		}

		cmd_buf
	}

	/// Ends the small GPU operation commandbuffer and sends the commands to the GPU.
	fn end_single_time_commands(&self, cmd_buf: vk::CommandBuffer)
	{
		unsafe {
			self.device.end_command_buffer(cmd_buf).expect("End commandbuffer");
		}

		let submit_info = vk::SubmitInfo {
			s_type: vk::StructureType::SubmitInfo,
			p_next: ptr::null(),
			wait_semaphore_count: 0,
			p_wait_semaphores: ptr::null(),
			p_wait_dst_stage_mask: ptr::null(),
			command_buffer_count: 1,
			p_command_buffers: &cmd_buf,
			signal_semaphore_count: 0,
			p_signal_semaphores: ptr::null(),
		};
		unsafe {
			self.device
				.queue_submit(self.graphics_queue, &[submit_info], vk::Fence::null())
				.expect("queue submit failed.");
			self.device.queue_wait_idle(self.graphics_queue).expect("queue wait failed.");
			self.device.free_command_buffers(self.commandpool, &[cmd_buf]);
		}
	}

	/// Creates a vk::Buffer based on the requirements.
	fn create_buffer(
		&self, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, buffersize: vk::DeviceSize
	) -> (vk::Buffer, vk::DeviceMemory)
	{
		let bufferinfo = vk::BufferCreateInfo {
			s_type: vk::StructureType::BufferCreateInfo,
			p_next: ptr::null(),
			flags: vk::BufferCreateFlags::empty(),
			size: buffersize,
			usage: usage,
			sharing_mode: vk::SharingMode::Exclusive,
			queue_family_index_count: 0,
			p_queue_family_indices: ptr::null(),
		};

		let buffer;
		unsafe {
			buffer = self.device.create_buffer(&bufferinfo, None).expect("Failed to create buffer");
		}

		let mem_req = self.device.get_buffer_memory_requirements(buffer);
		let alloc_info = vk::MemoryAllocateInfo {
			s_type: vk::StructureType::MemoryAllocateInfo,
			p_next: ptr::null(),
			allocation_size: mem_req.size,
			memory_type_index: self.find_memory_type(mem_req.memory_type_bits, properties),
		};
		let memory;
		unsafe {
			memory = self.device.allocate_memory(&alloc_info, None).expect("Failed to allocate buffer memory");

			self.device.bind_buffer_memory(buffer, memory, 0).expect("Failed to bind memory");
		}

		(buffer, memory)
	}

	/// Creates a vk::Buffer based on the requirements and fills it with the passed data.
	pub fn create_buffer_and_upload<T: Copy>(
		&self, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, upload_data: &[T],
		optimal_layout: bool,
	) -> (vk::Buffer, vk::DeviceMemory)
	{
		let mut buffer;
		let mut memory;
		let buffersize: vk::DeviceSize = (size_of::<T>() * upload_data.len()) as u64;

		// Create a temporary staging buffer
		if optimal_layout
		{
			debug_assert!((properties & vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT) == vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT);

			let (buf, mem) = self.create_buffer(
				vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
				vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
				buffersize,
			);
			buffer = buf;
			memory = mem;
		// Create the actual buffer
		}
		else
		{
			debug_assert!(
				(properties & (vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT)) ==
					(vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT)
			);

			let (buf, mem) = self.create_buffer(usage, properties, buffersize);
			buffer = buf;
			memory = mem;
		}

		// Upload data to the buffer we just created
		unsafe {
			let mem_ptr = self.device
				.map_memory(memory, 0, buffersize, vk::MemoryMapFlags::empty())
				.expect("Failed to map index memory");
			let mut mem_align = Align::new(mem_ptr, align_of::<T>() as u64, buffersize);
			mem_align.copy_from_slice(upload_data);
			self.device.unmap_memory(memory);
		}

		// For optimal buffers: create a new buffer with the optimal layout and copy the staging
		// buffer into it
		if optimal_layout
		{
			let staging_buffer = buffer;
			let staging_memory = memory;

			// Create final buffer
			let (buf, mem) = self.create_buffer(vk::BUFFER_USAGE_TRANSFER_DST_BIT | usage, properties, buffersize);
			buffer = buf;
			memory = mem;

			// Copy contents
			let cmd_buf = self.begin_single_time_commands();
			let buffer_copy_region = vk::BufferCopy {
				size: buffersize,
				src_offset: 0,
				dst_offset: 0,
			};
			unsafe {
				self.device.cmd_copy_buffer(cmd_buf, staging_buffer, buffer, &[buffer_copy_region]);
			}
			self.end_single_time_commands(cmd_buf);

			// Free staging buffer
			unsafe {
				self.device.destroy_buffer(staging_buffer, None);
				self.device.free_memory(staging_memory, None);
			}
		}

		(buffer, memory)
	}

	/// Creates a vk::ShaderModule from the given path.
	///
	/// Note: The path must point to a .spv file.
	fn load_shader(&self, path: &str) -> vk::ShaderModule
	{
		let spv_file = File::open(Path::new(path)).expect("Could not find spv file");
		let shader_bytes: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();
		let shader_info = vk::ShaderModuleCreateInfo {
			s_type: vk::StructureType::ShaderModuleCreateInfo,
			p_next: ptr::null(),
			flags: Default::default(),
			code_size: shader_bytes.len(),
			p_code: shader_bytes.as_ptr() as *const u32,
		};
		let shader_module;
		unsafe {
			shader_module = self.device.create_shader_module(&shader_info, None).expect("Shader module error");
		}
		shader_module
	}

	/// Creates a texture, view and sampler based on the passed options.
	///
	/// A vk::Buffer can optionally be passed to fill the texture with initial data.
	fn create_texture(
		&self, texture_dimensions: vk::Extent3D, texture_type: vk::ImageType, texture_view_type: vk::ImageViewType,
		texture_format: vk::Format, texture_aspect_mask: vk::ImageAspectFlags, mut texture_usage: vk::ImageUsageFlags,
		initial_access_mask: vk::AccessFlags, initial_layout: vk::ImageLayout, initial_stage: vk::PipelineStageFlags,
		upload_buffer: Option<vk::Buffer>,
	) -> Texture
	{
		// In case we need to upload to the texture, mark it for transfer dst
		if upload_buffer.is_some()
		{
			texture_usage |= vk::IMAGE_USAGE_TRANSFER_DST_BIT;
		}

		let texture_create_info = vk::ImageCreateInfo {
			s_type: vk::StructureType::ImageCreateInfo,
			p_next: ptr::null(),
			flags: Default::default(),
			image_type: texture_type,
			format: texture_format,
			extent: texture_dimensions,
			mip_levels: 1,
			array_layers: 1,
			samples: vk::SAMPLE_COUNT_1_BIT,
			tiling: vk::ImageTiling::Optimal,
			usage: texture_usage,
			sharing_mode: vk::SharingMode::Exclusive,
			queue_family_index_count: 0,
			p_queue_family_indices: ptr::null(),
			initial_layout: vk::ImageLayout::Undefined,
		};
		let texture_image;
		unsafe {
			texture_image = self.device.create_image(&texture_create_info, None).unwrap();
		}

		let texture_memory_req = self.device.get_image_memory_requirements(texture_image);
		let texture_allocate_info = vk::MemoryAllocateInfo {
			s_type: vk::StructureType::MemoryAllocateInfo,
			p_next: ptr::null(),
			allocation_size: texture_memory_req.size,
			memory_type_index: self.find_memory_type(
				texture_memory_req.memory_type_bits,
				vk::MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
			),
		};
		let texture_memory;
		unsafe {
			texture_memory = self.device.allocate_memory(&texture_allocate_info, None).unwrap();
			self.device.bind_image_memory(texture_image, texture_memory, 0).expect("Failed to bind memory");
		}

		// Transition the Image and potentially upload
		let cmd_buf = self.begin_single_time_commands();
		match upload_buffer
		{
			// In case we need to upload some texture data
			Some(image_upload_buffer) =>
			{
				// First transition the Image to TransferDstOptimal
				let texture_barrier = vk::ImageMemoryBarrier {
					s_type: vk::StructureType::ImageMemoryBarrier,
					p_next: ptr::null(),
					src_access_mask: Default::default(),
					dst_access_mask: vk::ACCESS_TRANSFER_WRITE_BIT,
					old_layout: vk::ImageLayout::Undefined,
					new_layout: vk::ImageLayout::TransferDstOptimal,
					src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
					dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
					image: texture_image,
					subresource_range: vk::ImageSubresourceRange {
						aspect_mask: texture_aspect_mask,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					},
				};
				unsafe {
					self.device.cmd_pipeline_barrier(
						cmd_buf,
						vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
						vk::PIPELINE_STAGE_TRANSFER_BIT,
						vk::DependencyFlags::empty(),
						&[],
						&[],
						&[texture_barrier],
					);
				}
				// Copy buffer data to image
				let buffer_copy_region = vk::BufferImageCopy {
					buffer_offset: 0,
					buffer_row_length: 0,
					buffer_image_height: 0,
					image_subresource: vk::ImageSubresourceLayers {
						aspect_mask: texture_aspect_mask,
						mip_level: 0,
						base_array_layer: 0,
						layer_count: 1,
					},
					image_extent: texture_dimensions,
					image_offset: vk::Offset3D {
						x: 0,
						y: 0,
						z: 0,
					},
				};
				unsafe {
					self.device.cmd_copy_buffer_to_image(
						cmd_buf,
						image_upload_buffer,
						texture_image,
						vk::ImageLayout::TransferDstOptimal,
						&[buffer_copy_region],
					);
				}
				// Finally transition the Image to the correct layout
				let texture_barrier = vk::ImageMemoryBarrier {
					s_type: vk::StructureType::ImageMemoryBarrier,
					p_next: ptr::null(),
					src_access_mask: vk::ACCESS_TRANSFER_WRITE_BIT,
					dst_access_mask: initial_access_mask,
					old_layout: vk::ImageLayout::TransferDstOptimal,
					new_layout: initial_layout,
					src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
					dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
					image: texture_image,
					subresource_range: vk::ImageSubresourceRange {
						aspect_mask: texture_aspect_mask,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					},
				};
				unsafe {
					self.device.cmd_pipeline_barrier(
						cmd_buf,
						vk::PIPELINE_STAGE_TRANSFER_BIT,
						initial_stage,
						vk::DependencyFlags::empty(),
						&[],
						&[],
						&[texture_barrier],
					);
				}
			}
			// Else, just transition the Image
			_ =>
			{
				let texture_barrier = vk::ImageMemoryBarrier {
					s_type: vk::StructureType::ImageMemoryBarrier,
					p_next: ptr::null(),
					src_access_mask: Default::default(),
					dst_access_mask: initial_access_mask,
					old_layout: vk::ImageLayout::Undefined,
					new_layout: initial_layout,
					src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
					dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
					image: texture_image,
					subresource_range: vk::ImageSubresourceRange {
						aspect_mask: texture_aspect_mask,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					},
				};
				unsafe {
					self.device.cmd_pipeline_barrier(
						cmd_buf,
						vk::PIPELINE_STAGE_TOP_OF_PIPE_BIT,
						initial_stage,
						vk::DependencyFlags::empty(),
						&[],
						&[],
						&[texture_barrier],
					);
				}
			}
		}
		self.end_single_time_commands(cmd_buf);

		// Create texture image view
		let tex_image_view_info = vk::ImageViewCreateInfo {
			s_type: vk::StructureType::ImageViewCreateInfo,
			p_next: ptr::null(),
			flags: Default::default(),
			view_type: texture_view_type,
			format: texture_create_info.format,
			components: vk::ComponentMapping {
				r: vk::ComponentSwizzle::R,
				g: vk::ComponentSwizzle::G,
				b: vk::ComponentSwizzle::B,
				a: vk::ComponentSwizzle::A,
			},
			subresource_range: vk::ImageSubresourceRange {
				aspect_mask: texture_aspect_mask,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			},
			image: texture_image,
		};
		let texture_view;
		unsafe {
			texture_view = self.device.create_image_view(&tex_image_view_info, None).unwrap();
		}

		// Create sampler
		let sampler_info = vk::SamplerCreateInfo {
			s_type: vk::StructureType::SamplerCreateInfo,
			p_next: ptr::null(),
			flags: Default::default(),
			mag_filter: vk::Filter::Linear,
			min_filter: vk::Filter::Linear,
			mipmap_mode: vk::SamplerMipmapMode::Linear,
			address_mode_u: vk::SamplerAddressMode::MirroredRepeat,
			address_mode_v: vk::SamplerAddressMode::MirroredRepeat,
			address_mode_w: vk::SamplerAddressMode::MirroredRepeat,
			mip_lod_bias: 0.0,
			min_lod: 0.0,
			max_lod: 0.0,
			anisotropy_enable: 0,
			max_anisotropy: 1.0,
			border_color: vk::BorderColor::FloatOpaqueWhite,
			compare_enable: 0,
			compare_op: vk::CompareOp::Never,
			unnormalized_coordinates: 0,
		};
		let sampler;
		unsafe {
			sampler = self.device.create_sampler(&sampler_info, None).unwrap();
		}

		Texture {
			image: texture_image,
			memory: texture_memory,
			view: texture_view,
			sampler: sampler,
			current_access_mask: initial_access_mask,
			current_layout: initial_layout,
			current_stage: initial_stage,
		}
	}

	/// Loads the image given by the path into read only texture.
	///
	/// Note: The caller is responsible for cleaning up the returned vulkan types.
	pub fn load_image(&self, path: &str) -> Texture
	{
		// Load the image data into a vk::Buffer
		let image = image::open(path).unwrap().to_rgba();
		let image_extent;
		{
			let image_dims = image.dimensions();
			image_extent = vk::Extent3D {
				width: image_dims.0,
				height: image_dims.1,
				depth: 1,
			};
		}
		let image_data = image.into_raw();
		let (image_buffer, image_memory) = self.create_buffer_and_upload(
			vk::BUFFER_USAGE_TRANSFER_SRC_BIT,
			vk::MEMORY_PROPERTY_HOST_VISIBLE_BIT | vk::MEMORY_PROPERTY_HOST_COHERENT_BIT,
			&image_data,
			false,
		);

		// Create a texture from the buffer data
		let texture = self.create_texture(
			image_extent,
			vk::ImageType::Type2d,
			vk::ImageViewType::Type2d,
			vk::Format::R8g8b8a8Unorm,
			vk::IMAGE_ASPECT_COLOR_BIT,
			vk::IMAGE_USAGE_SAMPLED_BIT,
			vk::ACCESS_SHADER_READ_BIT,
			vk::ImageLayout::ShaderReadOnlyOptimal,
			vk::PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
			Some(image_buffer),
		);

		// Texture now holds the data, can delete image buffer and memory
		unsafe {
			self.device.destroy_buffer(image_buffer, None);
			self.device.free_memory(image_memory, None);
		}

		texture
	}

	/// Transitions a Texture from its current access_mask/layout/pipeline_stage to the passed
	/// values.
	///
	/// This will use a single time command buffer unless one is passed to the function.
	pub fn transition_texture(
		&self, texture: &mut Texture, new_access_mask: vk::AccessFlags, new_layout: vk::ImageLayout,
		new_stage: vk::PipelineStageFlags, opt_cmd_buf: Option<vk::CommandBuffer>,
	)
	{
		// Skip if there's nothing to do
		if texture.current_access_mask == new_access_mask && texture.current_layout == new_layout &&
			texture.current_stage == new_stage
		{
			return;
		}

		let texture_barrier = vk::ImageMemoryBarrier {
			s_type: vk::StructureType::ImageMemoryBarrier,
			p_next: ptr::null(),
			src_access_mask: texture.current_access_mask,
			dst_access_mask: new_access_mask,
			old_layout: texture.current_layout,
			new_layout: new_layout,
			src_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
			dst_queue_family_index: vk::VK_QUEUE_FAMILY_IGNORED,
			image: texture.image,
			subresource_range: vk::ImageSubresourceRange {
				aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			},
		};

		match opt_cmd_buf
		{
			Some(cmd_buf) =>
			unsafe {
				self.device.cmd_pipeline_barrier(
					cmd_buf,
					texture.current_stage,
					new_stage,
					vk::DependencyFlags::empty(),
					&[],
					&[],
					&[texture_barrier],
				);
			},
			None =>
			{
				let cmd_buf = self.begin_single_time_commands();
				unsafe {
					self.device.cmd_pipeline_barrier(
						cmd_buf,
						texture.current_stage,
						new_stage,
						vk::DependencyFlags::empty(),
						&[],
						&[],
						&[texture_barrier],
					);
				}
				self.end_single_time_commands(cmd_buf);
			}
		}
		texture.current_access_mask = new_access_mask;
		texture.current_layout = new_layout;
		texture.current_stage = new_stage;
	}
}

impl Drop for RenderState
{
	/// Drops the Renderstate. This destroys the pools, device and instance.
	///
	/// It is the last ting to drop before ending the program, as any other Vulkan state must hav
	/// been freed at this point.
	fn drop(&mut self)
	{
		// We must have the only reference to device at this point
		debug_assert!(1 == Rc::strong_count(&self.device));

		unsafe {
			// Always wait for device idle
			self.device.device_wait_idle().unwrap();

			self.device.destroy_command_pool(self.commandpool, None);
			self.device.destroy_device(None);
			if cfg!(feature = "debug_layer")
			{
				match self.debug_report_loader
				{
					Some(ref loader) => match self.debug_callback
					{
						Some(callback) => loader.destroy_debug_report_callback_ext(callback, None),
						None => panic!("Debug callback is None!"),
					},
					None => panic!("Debug report loader is None!"),
				}
			}
			self.instance.destroy_instance(None);
		}
	}
}
