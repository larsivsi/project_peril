use config::Config;

use ash::{Device, Entry, Instance};
use ash::vk;
use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
use ash::extensions::{DebugReport, Surface, Swapchain, XlibSurface};
use std;
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::ptr;
use std::rc::Rc;
use winit;
use winit::EventsLoop;
use winit::Window;

pub struct RenderState {
    // Vulkan device
    entry: Entry<V1_0>,
    instance: Instance<V1_0>,
    debug_report_loader: Option<DebugReport>,
    debug_callback: Option<vk::DebugReportCallbackEXT>,
    pdevice: vk::PhysicalDevice,
    pub device: Rc<Device<V1_0>>,
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_family_index: u32,

    // Window
    pub event_loop: EventsLoop,
    window: Window,

    // Pools
    commandpool: vk::CommandPool,
}

impl RenderState {
    /// Lists the extensions required by the application.
    fn extension_names() -> Vec<*const i8> {
        let mut extensions = vec![Surface::name().as_ptr(), XlibSurface::name().as_ptr()];
        #[cfg(debug_assertions)]
        {
            extensions.push(DebugReport::name().as_ptr());
        }
        extensions
    }

    /// Creates a Vulkan instance.
    ///
    /// * `cfg`    The application config.
    /// * `entry`  The ash entrypoint.
    fn create_instance(cfg: &Config, entry: &Entry<V1_0>) -> Instance<V1_0> {
        // Application info
        let app_name = CString::new(cfg.app_name).unwrap();
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
        // Only enable debug layers in debug builds
        #[cfg(debug_assertions)]
        {
            println!("Debug layers:");
            let available_layers = entry.enumerate_instance_layer_properties().unwrap();
            for layer in available_layers.iter() {
                let layer_name;
                unsafe {
                    layer_name = CStr::from_ptr(layer.layer_name.as_ptr());
                }
                println!("Found layer {:?}", layer_name);
                for req_layer in requested_layers.iter() {
                    if layer_name == req_layer.as_c_str() {
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
            instance = entry.create_instance(&create_info, None).expect(
                "Instance creation error",
            );
        }

        instance
    }

    /// Debug layer callback function.
    ///
    /// This function is called from the debug layer if an issue is identified.
    unsafe extern "system" fn vulkan_debug_callback(
        _: vk::DebugReportFlagsEXT,
        _: vk::DebugReportObjectTypeEXT,
        _: vk::uint64_t,
        _: vk::size_t,
        _: vk::int32_t,
        _: *const vk::c_char,
        p_message: *const vk::c_char,
        _: *mut vk::c_void,
    ) -> u32 {
        println!("{:?}", CStr::from_ptr(p_message));
        1
    }

    /// Sets up the debug report layer and callback.
    ///
    /// * `entry`     The ash entrypoint.
    /// * `instance`  The Vulkan instance.
    fn setup_debug_callback(
        entry: &Entry<V1_0>,
        instance: &Instance<V1_0>,
    ) -> (DebugReport, vk::DebugReportCallbackEXT) {
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
            debug_callback = debug_report_loader
                .create_debug_report_callback_ext(&debug_info, None)
                .unwrap();
        }

        (debug_report_loader, debug_callback)
    }

    /// Selects a physical device (and queue index) for the Vulkan instance.
    ///
    /// * `instance`  The Vulkan instance.
    fn pick_physical_device(instance: &Instance<V1_0>) -> (vk::PhysicalDevice, u32) {
        let pdevices = instance.enumerate_physical_devices().expect(
            "Failed to find GPU with Vulkan support",
        );
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
                        match supports_graphics {
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
    ///
    /// * `instance`            The Vulkan instance.
    /// * `pdevice`             The Vulkan physical device.
    /// * `queue_family_index`  The queue index for this physical device.
    fn create_logical_device(
        instance: &Instance<V1_0>,
        pdevice: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Device<V1_0> {
        let queue_priorities = [1.0]; // One queue of priority 1.0
        let queue_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DeviceQueueCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            queue_family_index: queue_family_index,
            p_queue_priorities: queue_priorities.as_ptr(),
            queue_count: queue_priorities.len() as u32,
        };
        let device_extension_names_raw = [Swapchain::name().as_ptr()]; //VK_KHR_swapchain
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
            device = instance
                .create_device(pdevice, &device_create_info, None)
                .expect("Failed to create logical device");
        }

        device
    }

    /// Creates various pools required by the RenderState.
    ///
    /// * `device`              The logical Vulkan device.
    /// * `queue_family_index`  The queue index for this physical device.
    fn create_pools(device: &Device<V1_0>, queue_family_index: u32) -> (vk::CommandPool) {
        let cmd_pool_create_info = vk::CommandPoolCreateInfo {
            s_type: vk::StructureType::CommandPoolCreateInfo,
            p_next: ptr::null(),
            flags: vk::COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
            queue_family_index: queue_family_index,
        };
        let commandpool;
        unsafe {
            commandpool = device
                .create_command_pool(&cmd_pool_create_info, None)
                .unwrap()
        }

        (commandpool)
    }

    /// Initializes the RenderState based in the passed Config.
    ///
    /// * `cfg`  The config for the RenderState.
    pub fn init(cfg: &Config) -> RenderState {
        // Window and event handler
        let event_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new()
            .with_title(format!("{} {}", cfg.app_name, cfg.version_to_string()))
            .with_dimensions(cfg.window_dimensions.0, cfg.window_dimensions.1)
            .build(&event_loop)
            .unwrap();

        // ash entry point
        let entry: Entry<V1_0> = Entry::new().unwrap();

        // Vulkan init
        let instance = RenderState::create_instance(&cfg, &entry);
        let mut debug_report_loader = None;
        let mut debug_callback = None;
        #[cfg(debug_assertions)]
        {
            let (loader, callback) = RenderState::setup_debug_callback(&entry, &instance);
            debug_report_loader = Some(loader);
            debug_callback = Some(callback);
        }
        let (pdevice, queue_family_index) = RenderState::pick_physical_device(&instance);
        let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);
        let device = RenderState::create_logical_device(&instance, pdevice, queue_family_index);

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

            // Window
            event_loop: event_loop,
            window: window,

            // Pools
            commandpool: commandpool,
        }
    }

    /// Returns a suitable memory type for the requirements based in the physical Vulkan device.
    ///
    /// * `mem_type_bits`  A bitmask of the requested memory types.
    /// * `properties`     The memory properties required for the requested memory.
    fn find_memory_type(&self, mem_type_bits: u32, properties: vk::MemoryPropertyFlags) -> u32 {
        for (idx, mem_type) in self.device_memory_properties
            .memory_types
            .iter()
            .enumerate()
        {
            if mem_type_bits & (1 << idx) != 0 &&
                (mem_type.property_flags & properties) == properties
            {
                return idx as u32;
            }
        }
        panic!("Cannot find memory type!");
    }

    /// Creates a vk::Buffer based on the requirements.
    ///
    /// * `size`        The size of the resulting buffer.
    /// * `usage`       Usage bits for the resulting buffer.
    /// * `properties`  Memory properties required for the buffer.
    pub fn create_buffer(
        &self,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> (vk::Buffer, vk::DeviceMemory) {
        let bufferinfo = vk::BufferCreateInfo {
            s_type: vk::StructureType::BufferCreateInfo,
            p_next: ptr::null(),
            flags: vk::BufferCreateFlags::empty(),
            size: size,
            usage: usage,
            sharing_mode: vk::SharingMode::Exclusive,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };
        let buffer;
        unsafe {
            buffer = self.device.create_buffer(&bufferinfo, None).expect(
                "Failed to create buffer",
            );
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
            memory = self.device.allocate_memory(&alloc_info, None).expect(
                "Failed to allocate buffer memory",
            );

            self.device.bind_buffer_memory(buffer, memory, 0).expect(
                "Failed to bind memory",
            );
        }

        (buffer, memory)
    }

    /// Creates a vk::ShaderModule from the given path.
    ///
    /// Note: The path must point to a .spv file.
    ///
    /// * `path`  Path to the shader.
    fn load_shader(&self, path: &str) -> vk::ShaderModule {
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
            shader_module = self.device
                .create_shader_module(&shader_info, None)
                .expect("Shader module error");
        }
        shader_module
    }
}

impl Drop for RenderState {
    /// Drops the Renderstate. This destroys the pools, device and instance.
    ///
    /// It is the last ting to drop before ending the program, as any other Vulkan state must hav
    /// been freed at this point.
    fn drop(&mut self) {
        // We must have the only reference to device at this point
        debug_assert!(1 == Rc::strong_count(&self.device));

        unsafe {
            // Always wait for device idle
            self.device.device_wait_idle().unwrap();

            self.device.destroy_command_pool(self.commandpool, None);
            self.device.destroy_device(None);
            #[cfg(debug_assertions)]
            {
                match self.debug_report_loader {
                    Some(ref loader) => {
                        match self.debug_callback {
                            Some(callback) => {
                                loader.destroy_debug_report_callback_ext(callback, None)
                            }
                            None => panic!("Debug callback is None!"),
                        }
                    }
                    None => panic!("Debug report loader is None!"),
                }
            }
            self.instance.destroy_instance(None);
        }
    }
}

pub struct PresentState {
    // Surface
    surface_loader: Surface,
    surface: vk::SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,

    // Present queue
    present_queue: vk::Queue,
    image_available_sem: vk::Semaphore,
    rendering_finished_sem: vk::Semaphore,

    swapchain_loader: Swapchain,

    // Swapchain
    swapchain: vk::SwapchainKHR,
    //presentable images for the screen
    present_image_views: Vec<vk::ImageView>,
    renderpass: vk::RenderPass,
    //one framebuffer per image
    framebuffers: Vec<vk::Framebuffer>,
    pipeline_layout: vk::PipelineLayout,
    viewport: vk::Viewport,
    scissor: vk::Rect2D,
    pipeline: vk::Pipeline,
    //one commandbuffer per image
    commandbuffers: Vec<vk::CommandBuffer>,

    // The current idx
    current_present_idx: usize,

    // Keep a pointer to the device for cleanup
    device: Rc<Device<V1_0>>,
}

impl PresentState {
    /// Creates an X11 surface.
    ///
    /// * `entry`     The ash entrypoint.
    /// * `instance`  The Vulkan instance.
    /// * `window`    The window to create the surface for.
    fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
        entry: &E,
        instance: &I,
        window: &winit::Window,
    ) -> Result<vk::SurfaceKHR, vk::Result> {
        use winit::os::unix::WindowExt;
        let x11_display = window.get_xlib_display().unwrap();
        let x11_window = window.get_xlib_window().unwrap();
        let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
            s_type: vk::StructureType::XlibSurfaceCreateInfoKhr,
            p_next: ptr::null(),
            flags: Default::default(),
            window: x11_window as vk::Window,
            dpy: x11_display as *mut vk::Display,
        };
        let xlib_surface_loader =
            XlibSurface::new(entry, instance).expect("Unable to load xlib surface");
        let result;
        unsafe {
            result = xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None);
        }
        result
    }

    /// Creates a vk::Swapchain and a vk::Rect2D for the current RenderState and surface.
    ///
    /// Swapchain is used to queue and present stuff to the screen.
    ///
    /// * `rs`                The RenderState.
    /// * `surface_loader`    The surface loader.
    /// * `surface`           The surface for the swapchain.
    /// * `surface_format`    The format of the surface.
    /// * `old_swapchain`     The swapchain to re-use. Note that this must be a valid (i.e. not
    ///                       destroyed) swapchain.
    /// * `swapchain_loader`  The swapchain loader.
    fn create_swapchain(
        rs: &RenderState,
        surface_loader: &Surface,
        surface: &vk::SurfaceKHR,
        surface_format: &vk::SurfaceFormatKHR,
        old_swapchain: vk::SwapchainKHR,
        swapchain_loader: &Swapchain,
    ) -> (vk::SwapchainKHR, vk::Rect2D) {
        let surface_capabilities = surface_loader
            .get_physical_device_surface_capabilities_khr(rs.pdevice, *surface)
            .unwrap();
        //TODO double-buffering for now
        let mut desired_image_count = 2;
        if surface_capabilities.max_image_count > 0 &&
            desired_image_count > surface_capabilities.max_image_count
        {
            desired_image_count = surface_capabilities.max_image_count;
        }
        let pre_transform = if surface_capabilities.supported_transforms.subset(
            vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
        )
        {
            vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR
        } else {
            surface_capabilities.current_transform
        };
        let present_modes = surface_loader
            .get_physical_device_surface_present_modes_khr(rs.pdevice, *surface)
            .unwrap();
        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::Mailbox)
            .unwrap_or(vk::PresentModeKHR::Fifo);
        let swapchain_create_info = vk::SwapchainCreateInfoKHR {
            s_type: vk::StructureType::SwapchainCreateInfoKhr,
            p_next: ptr::null(),
            flags: Default::default(),
            surface: *surface,
            min_image_count: desired_image_count,
            image_color_space: surface_format.color_space,
            image_format: surface_format.format,
            image_extent: surface_capabilities.current_extent.clone(),
            image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            image_sharing_mode: vk::SharingMode::Exclusive,
            pre_transform: pre_transform,
            composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            present_mode: present_mode,
            clipped: 1,
            old_swapchain: old_swapchain,
            image_array_layers: 1,
            p_queue_family_indices: ptr::null(),
            queue_family_index_count: 0,
        };
        let swapchain;
        unsafe {
            swapchain = swapchain_loader
                .create_swapchain_khr(&swapchain_create_info, None)
                .unwrap();
        }

        (
            swapchain,
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: surface_capabilities.current_extent.clone(),
            },
        )
    }

    /// Creates a Vec of vk::ImageViews for the presentable images in the swapchain.
    ///
    /// This will create two imageviews for double-buffering, three imageviews for
    /// tripple-buffering etc.
    ///
    /// * `rs`                The RenderState.
    /// * `surface_format`    The format of the surface.
    /// * `swapchain_loader`  The swapchain loader.
    /// * `swapchain`         The swapchain to create image views for.
    fn create_imageviews(
        rs: &RenderState,
        surface_format: &vk::SurfaceFormatKHR,
        swapchain_loader: &Swapchain,
        swapchain: vk::SwapchainKHR,
    ) -> Vec<vk::ImageView> {
        let present_images = swapchain_loader
            .get_swapchain_images_khr(swapchain)
            .unwrap();
        let present_image_views: Vec<vk::ImageView> = present_images
            .iter()
            .map(|&image| {
                let create_view_info = vk::ImageViewCreateInfo {
                    s_type: vk::StructureType::ImageViewCreateInfo,
                    p_next: ptr::null(),
                    flags: Default::default(),
                    view_type: vk::ImageViewType::Type2d,
                    format: surface_format.format,
                    components: vk::ComponentMapping {
                        r: vk::ComponentSwizzle::R,
                        g: vk::ComponentSwizzle::G,
                        b: vk::ComponentSwizzle::B,
                        a: vk::ComponentSwizzle::A,
                    },
                    subresource_range: vk::ImageSubresourceRange {
                        aspect_mask: vk::IMAGE_ASPECT_COLOR_BIT,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    },
                    image: image,
                };
                let result;
                unsafe {
                    result = rs.device
                        .create_image_view(&create_view_info, None)
                        .unwrap()
                }
                result
            })
            .collect();

        present_image_views
    }

    /// Creates a presentable renderpass.
    ///
    /// Produces a color-only renderpass, perfect for direct drawing.
    ///
    /// * `rs`              The RenderState.
    /// * `surface_format`  The format of the surface.
    fn create_renderpass(
        rs: &RenderState,
        surface_format: &vk::SurfaceFormatKHR,
    ) -> vk::RenderPass {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: surface_format.format,
                flags: vk::AttachmentDescriptionFlags::empty(),
                samples: vk::SAMPLE_COUNT_1_BIT,
                load_op: vk::AttachmentLoadOp::Clear,
                store_op: vk::AttachmentStoreOp::Store,
                stencil_load_op: vk::AttachmentLoadOp::DontCare,
                stencil_store_op: vk::AttachmentStoreOp::DontCare,
                initial_layout: vk::ImageLayout::Undefined,
                final_layout: vk::ImageLayout::PresentSrcKhr,
            },
        ];
        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::ColorAttachmentOptimal,
        };
        let dependency = vk::SubpassDependency {
            dependency_flags: Default::default(),
            src_subpass: vk::VK_SUBPASS_EXTERNAL,
            dst_subpass: Default::default(),
            src_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
            src_access_mask: Default::default(),
            dst_access_mask: vk::ACCESS_COLOR_ATTACHMENT_READ_BIT |
                vk::ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
            dst_stage_mask: vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
        };
        let subpass = vk::SubpassDescription {
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_depth_stencil_attachment: ptr::null(),
            flags: Default::default(),
            pipeline_bind_point: vk::PipelineBindPoint::Graphics,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            p_resolve_attachments: ptr::null(),
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };
        let renderpass_create_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RenderPassCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            attachment_count: renderpass_attachments.len() as u32,
            p_attachments: renderpass_attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: 1,
            p_dependencies: &dependency,
        };
        let renderpass;
        unsafe {
            renderpass = rs.device
                .create_render_pass(&renderpass_create_info, None)
                .unwrap();
        }

        renderpass
    }

    /// Creates a pipeline for the given presentable renderpass.
    ///
    /// Very straigt forward pipeline: Loads some hard-coded shaders that will draw a triangle.
    ///
    /// * `rs`            The RenderState.
    /// * `surface_size`  The size of the surface to render to.
    /// * `renderpass`    The renderpass to produce the pipeline for (these have to match).
    fn create_pipeline(
        rs: &RenderState,
        surface_size: vk::Rect2D,
        renderpass: vk::RenderPass,
    ) -> (vk::PipelineLayout, vk::Viewport, vk::Rect2D, vk::Pipeline) {
        let layout_create_info = vk::PipelineLayoutCreateInfo {
            s_type: vk::StructureType::PipelineLayoutCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            set_layout_count: 0,
            p_set_layouts: ptr::null(),
            push_constant_range_count: 0,
            p_push_constant_ranges: ptr::null(),
        };

        let pipeline_layout;
        unsafe {
            pipeline_layout = rs.device
                .create_pipeline_layout(&layout_create_info, None)
                .unwrap();
        }

        let vertex_shader_module = RenderState::load_shader(rs, "shaders/vertex.spv");
        let fragment_shader_module = RenderState::load_shader(rs, "shaders/fragment.spv");

        let shader_entry_name = CString::new("main").unwrap();
        let shader_stage_create_infos = [
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PipelineShaderStageCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                module: vertex_shader_module,
                p_name: shader_entry_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::SHADER_STAGE_VERTEX_BIT,
            },
            vk::PipelineShaderStageCreateInfo {
                s_type: vk::StructureType::PipelineShaderStageCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                module: fragment_shader_module,
                p_name: shader_entry_name.as_ptr(),
                p_specialization_info: ptr::null(),
                stage: vk::SHADER_STAGE_FRAGMENT_BIT,
            },
        ];
        let vertex_input_binding_descriptions = [];
        let vertex_input_attribute_descriptions = [];
        let vertex_input_state_info = vk::PipelineVertexInputStateCreateInfo {
            s_type: vk::StructureType::PipelineVertexInputStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            vertex_attribute_description_count: vertex_input_attribute_descriptions.len() as u32,
            p_vertex_attribute_descriptions: vertex_input_attribute_descriptions.as_ptr(),
            vertex_binding_description_count: vertex_input_binding_descriptions.len() as u32,
            p_vertex_binding_descriptions: vertex_input_binding_descriptions.as_ptr(),
        };
        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo {
            s_type: vk::StructureType::PipelineInputAssemblyStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            primitive_restart_enable: 0,
            topology: vk::PrimitiveTopology::TriangleList,
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
            s_type: vk::StructureType::PipelineViewportStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            scissor_count: 1,
            p_scissors: &scissor,
            viewport_count: 1,
            p_viewports: &viewport,
        };
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo {
            s_type: vk::StructureType::PipelineRasterizationStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            cull_mode: vk::CULL_MODE_NONE,
            depth_bias_clamp: 0.0,
            depth_bias_constant_factor: 0.0,
            depth_bias_enable: 0,
            depth_bias_slope_factor: 0.0,
            depth_clamp_enable: 0,
            front_face: vk::FrontFace::CounterClockwise,
            line_width: 1.0,
            polygon_mode: vk::PolygonMode::Fill,
            rasterizer_discard_enable: 0,
        };
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo {
            s_type: vk::StructureType::PipelineMultisampleStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            rasterization_samples: vk::SAMPLE_COUNT_1_BIT,
            sample_shading_enable: 0,
            min_sample_shading: 0.0,
            p_sample_mask: ptr::null(),
            alpha_to_one_enable: 0,
            alpha_to_coverage_enable: 0,
        };
        let noop_stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::Keep,
            pass_op: vk::StencilOp::Keep,
            depth_fail_op: vk::StencilOp::Keep,
            compare_op: vk::CompareOp::Always,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };
        let depth_state_info = vk::PipelineDepthStencilStateCreateInfo {
            s_type: vk::StructureType::PipelineDepthStencilStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            depth_test_enable: 1,
            depth_write_enable: 1,
            depth_compare_op: vk::CompareOp::LessOrEqual,
            depth_bounds_test_enable: 0,
            stencil_test_enable: 0,
            front: noop_stencil_state.clone(),
            back: noop_stencil_state.clone(),
            max_depth_bounds: 1.0,
            min_depth_bounds: 0.0,
        };
        let color_blend_attachment_states = [
            vk::PipelineColorBlendAttachmentState {
                blend_enable: 0,
                src_color_blend_factor: vk::BlendFactor::SrcColor,
                dst_color_blend_factor: vk::BlendFactor::OneMinusDstColor,
                color_blend_op: vk::BlendOp::Add,
                src_alpha_blend_factor: vk::BlendFactor::Zero,
                dst_alpha_blend_factor: vk::BlendFactor::Zero,
                alpha_blend_op: vk::BlendOp::Add,
                color_write_mask: vk::ColorComponentFlags::all(),
            },
        ];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo {
            s_type: vk::StructureType::PipelineColorBlendStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            logic_op_enable: 0,
            logic_op: vk::LogicOp::Clear,
            attachment_count: color_blend_attachment_states.len() as u32,
            p_attachments: color_blend_attachment_states.as_ptr(),
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };
        let dynamic_state = [vk::DynamicState::Viewport, vk::DynamicState::Scissor];
        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo {
            s_type: vk::StructureType::PipelineDynamicStateCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
            dynamic_state_count: dynamic_state.len() as u32,
            p_dynamic_states: dynamic_state.as_ptr(),
        };
        let graphic_pipeline_info = vk::GraphicsPipelineCreateInfo {
            s_type: vk::StructureType::GraphicsPipelineCreateInfo,
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
            graphics_pipelines = rs.device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[graphic_pipeline_info],
                    None,
                )
                .expect("Unable to create graphics pipeline");

            // Graphics pipeline created, we no longer need the shader modules
            rs.device.destroy_shader_module(
                fragment_shader_module,
                None,
            );
            rs.device.destroy_shader_module(vertex_shader_module, None);
        }

        (pipeline_layout, viewport, scissor, graphics_pipelines[0])
    }

    /// Creates framebuffers for the presentable images, one per image.
    ///
    /// * `rs`                   The RenderState.
    /// * `surface_size`         The size of the surface to render to.
    /// * `present_image_views`  Imageviews to produce framebuffers for (one
    ///                          framebuffer per imageview).
    /// * `renderpass`           The renderpass to produce framebuffers for.
    fn create_framebuffers(
        rs: &RenderState,
        surface_size: vk::Rect2D,
        present_image_views: &Vec<vk::ImageView>,
        renderpass: vk::RenderPass,
    ) -> Vec<vk::Framebuffer> {
        let framebuffers: Vec<vk::Framebuffer> = present_image_views
            .iter()
            .map(|&present_image_view| {
                let framebuffer_attachments = [present_image_view];
                let frame_buffer_create_info = vk::FramebufferCreateInfo {
                    s_type: vk::StructureType::FramebufferCreateInfo,
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
                    framebuffer = rs.device
                        .create_framebuffer(&frame_buffer_create_info, None)
                        .unwrap();
                }
                framebuffer
            })
            .collect();

        framebuffers
    }

    /// Creates commandbuffers for the presentable images, one per image.
    ///
    /// * `rs`            The RenderState.
    /// * `framebuffers`  Framebuffers for the presentable images.
    fn create_commandbuffers(
        rs: &RenderState,
        framebuffers: &Vec<vk::Framebuffer>,
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::CommandBufferAllocateInfo,
            p_next: ptr::null(),
            command_buffer_count: framebuffers.len() as u32,
            command_pool: rs.commandpool,
            level: vk::CommandBufferLevel::Primary,
        };
        let command_buffers;
        unsafe {
            command_buffers = rs.device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .unwrap();
        }

        command_buffers
    }

    /// Initializes the PresentState based on a RenderState
    ///
    /// This will set up the swapchain, renderpass, etc.
    ///
    /// * `rs`  The RenderState.
    pub fn init(rs: &RenderState) -> PresentState {
        // Surface
        let surface_loader =
            Surface::new(&rs.entry, &rs.instance).expect("Unable to load the Surface extension");
        let surface = PresentState::create_surface(&rs.entry, &rs.instance, &rs.window).unwrap();
        debug_assert!(surface_loader.get_physical_device_surface_support_khr(
            rs.pdevice,
            rs.queue_family_index,
            surface,
        ));
        let surface_formats = surface_loader
            .get_physical_device_surface_formats_khr(rs.pdevice, surface)
            .unwrap();
        let surface_format = surface_formats
            .iter()
            .map(|sfmt| match sfmt.format {
                vk::Format::Undefined => {
                    vk::SurfaceFormatKHR {
                        format: vk::Format::B8g8r8Unorm,
                        color_space: sfmt.color_space,
                    }
                }
                _ => sfmt.clone(),
            })
            .nth(0)
            .expect("Unable to find suitable surface format.");

        // Present queue
        let present_queue;
        unsafe {
            present_queue = rs.device.get_device_queue(rs.queue_family_index, 0);
        }
        let sem_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SemaphoreCreateInfo,
            p_next: ptr::null(),
            flags: Default::default(),
        };
        let image_available_sem;
        let rendering_finished_sem;
        unsafe {
            image_available_sem = rs.device.create_semaphore(&sem_create_info, None).unwrap();
            rendering_finished_sem = rs.device.create_semaphore(&sem_create_info, None).unwrap();
        }

        let swapchain_loader =
            Swapchain::new(&rs.instance, rs.device.as_ref()).expect("Unable to load swapchain");

        let (swapchain, surface_size) = PresentState::create_swapchain(
            rs,
            &surface_loader,
            &surface,
            &surface_format,
            vk::SwapchainKHR::null(),
            &swapchain_loader,
        );
        let present_image_views =
            PresentState::create_imageviews(rs, &surface_format, &swapchain_loader, swapchain);
        let renderpass = PresentState::create_renderpass(rs, &surface_format);
        let (pipeline_layout, viewport, scissor, pipeline) =
            PresentState::create_pipeline(rs, surface_size, renderpass);
        let framebuffers =
            PresentState::create_framebuffers(rs, surface_size, &present_image_views, renderpass);
        let command_buffers = PresentState::create_commandbuffers(rs, &framebuffers);

        PresentState {
            // Surface
            surface_loader: surface_loader,
            surface: surface,
            surface_format: surface_format,

            // Present queue
            present_queue: present_queue,
            image_available_sem: image_available_sem,
            rendering_finished_sem: rendering_finished_sem,

            swapchain_loader: swapchain_loader,

            // Swapchain
            swapchain: swapchain,
            present_image_views: present_image_views,
            renderpass: renderpass,
            framebuffers: framebuffers,
            pipeline_layout: pipeline_layout,
            viewport: viewport,
            scissor: scissor,
            pipeline: pipeline,
            //one commandbuffer per image
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
    fn cleanup_swapchain(&mut self) {
        unsafe {
            // Always wait for device idle
            self.device.device_wait_idle().unwrap();

            for &framebuffer in self.framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(
                self.pipeline_layout,
                None,
            );

            self.device.destroy_render_pass(self.renderpass, None);

            for &image_view in self.present_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.swapchain_loader.destroy_swapchain_khr(
                self.swapchain,
                None,
            );
        }
    }

    /// Releases the old and creates a new swapchain.
    ///
    /// This function should be called when the presentable surface is resized, etc.
    ///
    /// * `rs`  The RenderState.
    fn recreate_swapchain(&mut self, rs: &RenderState) {
        self.cleanup_swapchain();

        let (swapchain, surface_size) = PresentState::create_swapchain(
            rs,
            &self.surface_loader,
            &self.surface,
            &self.surface_format,
            vk::SwapchainKHR::null(),
            &self.swapchain_loader,
        );
        self.swapchain = swapchain;
        let present_image_views = PresentState::create_imageviews(
            rs,
            &self.surface_format,
            &self.swapchain_loader,
            swapchain,
        );
        self.present_image_views = present_image_views;
        let renderpass = PresentState::create_renderpass(rs, &self.surface_format);
        self.renderpass = renderpass;
        let (pipeline_layout, viewport, scissor, pipeline) =
            PresentState::create_pipeline(rs, surface_size, renderpass);
        self.pipeline_layout = pipeline_layout;
        self.viewport = viewport;
        self.scissor = scissor;
        self.pipeline = pipeline;
        let framebuffers = PresentState::create_framebuffers(
            rs,
            surface_size,
            &self.present_image_views,
            renderpass,
        );
        self.framebuffers = framebuffers;
        let command_buffers = PresentState::create_commandbuffers(rs, &self.framebuffers);
        self.commandbuffers = command_buffers;
    }

    /// Starts a frame for the current swapchain. The returned commandbuffer should be used for
    /// rendering.
    ///
    /// On error (for example when the swapchain needs to be recreated), this function returns
    /// None, meaning that the current frame should be skipped.
    ///
    /// * `rs`  The RenderState.
    pub fn begin_frame(&mut self, rs: &RenderState) -> Option<vk::CommandBuffer> {
        let result;
        unsafe {
            result = self.swapchain_loader.acquire_next_image_khr(
                self.swapchain,
                std::u64::MAX,
                self.image_available_sem,
                vk::Fence::null(),
            );
        }

        match result {
            Ok(idx) => {
                self.current_present_idx = idx as usize;
            }
            Err(vkres) => {
                if vkres == vk::Result::ErrorOutOfDateKhr {
                    self.recreate_swapchain(rs);
                    return None;
                }
            }
        }

        // Begin commandbuffer
        let cmd_buf_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::CommandBufferBeginInfo,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::COMMAND_BUFFER_USAGE_SIMULTANEOUS_USE_BIT,
        };
        let cmd_buf = self.commandbuffers[self.current_present_idx];
        unsafe {
            rs.device
                .begin_command_buffer(cmd_buf, &cmd_buf_begin_info)
                .expect("Begin commandbuffer");
        }

        // Begin renderpass
        let clear_values =
            [
                vk::ClearValue::new_color(vk::ClearColorValue::new_float32([0.0, 0.0, 0.0, 1.0])),
            ];

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RenderPassBeginInfo,
            p_next: ptr::null(),
            render_pass: self.renderpass,
            framebuffer: self.framebuffers[self.current_present_idx],
            render_area: self.scissor,
            clear_value_count: clear_values.len() as u32,
            p_clear_values: clear_values.as_ptr(),
        };
        unsafe {
            // Start the render pass
            rs.device.cmd_begin_render_pass(
                cmd_buf,
                &render_pass_begin_info,
                vk::SubpassContents::Inline,
            );

            // Bind pipeline
            rs.device.cmd_bind_pipeline(
                cmd_buf,
                vk::PipelineBindPoint::Graphics,
                self.pipeline,
            );

            rs.device.cmd_set_viewport(cmd_buf, &[self.viewport]);
            rs.device.cmd_set_scissor(cmd_buf, &[self.scissor]);
        }

        Some(cmd_buf)
    }

    /// Ends the current frame and presents it.
    ///
    /// begin_frame() must have been called before this function.
    ///
    /// * `rs`  The RenderState.
    pub fn end_frame_and_present(&mut self, rs: &RenderState) {
        debug_assert!(self.current_present_idx < std::usize::MAX);

        let cmd_buf = self.commandbuffers[self.current_present_idx];
        unsafe {
            // End render pass and command buffer
            rs.device.cmd_end_render_pass(cmd_buf);
            rs.device.end_command_buffer(cmd_buf).expect(
                "End commandbuffer",
            );
        }

        // Send the work off to the GPU
        let fence_create_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FenceCreateInfo,
            p_next: ptr::null(),
            flags: vk::FenceCreateFlags::empty(),
        };
        let submit_fence;
        unsafe {
            submit_fence = rs.device.create_fence(&fence_create_info, None).expect(
                "Create fence failed.",
            );
        }
        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SubmitInfo,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: [self.image_available_sem].as_ptr(),
            p_wait_dst_stage_mask: [vk::PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT].as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &cmd_buf,
            signal_semaphore_count: 1,
            p_signal_semaphores: [self.rendering_finished_sem].as_ptr(),
        };
        unsafe {
            rs.device
                .queue_submit(self.present_queue, &[submit_info], submit_fence)
                .expect("queue submit failed.");
            rs.device
                .wait_for_fences(&[submit_fence], true, std::u64::MAX)
                .expect("Wait for fence failed.");
            rs.device.destroy_fence(submit_fence, None);
        }

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PresentInfoKhr,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.rendering_finished_sem,
            swapchain_count: 1,
            p_swapchains: &self.swapchain,
            p_image_indices: &(self.current_present_idx as u32),
            p_results: ptr::null_mut(),
        };
        unsafe {
            self.swapchain_loader
                .queue_present_khr(self.present_queue, &present_info)
                .unwrap();
        }

        // Make sure we call begin_frame() before calling this function again
        self.current_present_idx = std::usize::MAX;
    }
}

impl Drop for PresentState {
    /// Drops the PresentState. This destroys the swapchain and surface.
    fn drop(&mut self) {
        // We cannot have the last reference to device at this point
        debug_assert!(1 < Rc::strong_count(&self.device));

        // Already contains a device wait
        self.cleanup_swapchain();

        unsafe {
            self.device.destroy_semaphore(
                self.rendering_finished_sem,
                None,
            );
            self.device.destroy_semaphore(
                self.image_available_sem,
                None,
            );
            self.surface_loader.destroy_surface_khr(self.surface, None);
        }
    }
}
