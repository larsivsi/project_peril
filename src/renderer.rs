use config::Config;

use ash::{Device, Entry, Instance};
use ash::vk;
use ash::version::{V1_0, InstanceV1_0, DeviceV1_0, EntryV1_0};
use ash::extensions::{Surface, Swapchain, XlibSurface};
use std;
use std::ffi::CString;
use std::ptr;
use winit;
use winit::EventsLoop;
use winit::Window;

pub struct RenderState {
    instance: Instance<V1_0>,
    pub device: Device<V1_0>,
    pdevice: vk::PhysicalDevice,
    queue_family_index: u32,
    device_memory_properties: vk::PhysicalDeviceMemoryProperties,
    // window stuff
    pub event_loop: EventsLoop,
    window: Window,
    surface: vk::SurfaceKHR,
    // swapchain
    swapchain: vk::SwapchainKHR,
    present_images: Vec<vk::Image>,
    present_image_views: Vec<vk::ImageView>,
}

impl RenderState {
    fn extension_names() -> Vec<*const i8> {
        vec![Surface::name().as_ptr(), XlibSurface::name().as_ptr()]
    }

    // Creates X11 surface
    unsafe fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
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
        xlib_surface_loader.create_xlib_surface_khr(&x11_create_info, None)
    }

    // In case window changes dimensions (resized etc.)
    pub fn recreate_swapchain(&mut self) {}

    pub fn init(cfg: Config) -> RenderState {
        let dimensions = {
            let (width, height) = cfg.window_dimensions;
            [width, height]
        };

        let events_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new()
            .with_title(format!("{} {}", cfg.app_name, cfg.version_to_string()))
            .with_dimensions(dimensions[0], dimensions[1])
            .build(&events_loop)
            .unwrap();

        unsafe {
            let entry: Entry<V1_0> = Entry::new().unwrap();
            let app_name = CString::new(cfg.app_name).unwrap();
            let raw_name = app_name.as_ptr();

            let layer_names = [CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()];
            let layers_names_raw: Vec<*const i8> = layer_names
                .iter()
                .map(|raw_name| raw_name.as_ptr())
                .collect();
            let extension_names_raw = RenderState::extension_names();
            let appinfo = vk::ApplicationInfo {
                p_application_name: raw_name,
                s_type: vk::StructureType::ApplicationInfo,
                p_next: ptr::null(),
                application_version: cfg.app_version,
                p_engine_name: raw_name,
                engine_version: 0,
                api_version: vk_make_version!(1, 0, 57),
            };

            let create_info = vk::InstanceCreateInfo {
                s_type: vk::StructureType::InstanceCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                p_application_info: &appinfo,
                pp_enabled_layer_names: layers_names_raw.as_ptr(),
                enabled_layer_count: layers_names_raw.len() as u32,
                pp_enabled_extension_names: extension_names_raw.as_ptr(),
                enabled_extension_count: extension_names_raw.len() as u32,
            };

            let instance: Instance<V1_0> = entry.create_instance(&create_info, None).expect(
                "Instance creation error",
            );

            let surface = RenderState::create_surface(&entry, &instance, &window).unwrap();
            let pdevices = instance.enumerate_physical_devices().expect(
                "Physical device error",
            );
            let surface_loader =
                Surface::new(&entry, &instance).expect("Unable to load the Surface extension");

            let (pdevice, queue_family_index) = pdevices
                .iter()
                .map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .filter_map(|(index, ref info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.subset(vk::QUEUE_GRAPHICS_BIT) &&
                                    surface_loader.get_physical_device_surface_support_khr(
                                        *pdevice,
                                        index as u32,
                                        surface,
                                    );
                            match supports_graphic_and_surface {
                                true => Some((*pdevice, index)),
                                _ => None,
                            }
                        })
                        .nth(0)
                })
                .filter_map(|v| v)
                .nth(0)
                .expect("Couldn't find suitable device.");

            let queue_family_index = queue_family_index as u32;
            let device_extension_names_raw = [Swapchain::name().as_ptr()];
            let features = vk::PhysicalDeviceFeatures {
                shader_clip_distance: 1,
                ..Default::default()
            };

            let priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DeviceQueueCreateInfo,
                p_next: ptr::null(),
                flags: Default::default(),
                queue_family_index: queue_family_index as u32,
                p_queue_priorities: priorities.as_ptr(),
                queue_count: priorities.len() as u32,
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

            let device: Device<V1_0> = instance
                .create_device(pdevice, &device_create_info, None)
                .unwrap();
            let present_queue = device.get_device_queue(queue_family_index as u32, 0);

            let surface_formats = surface_loader
                .get_physical_device_surface_formats_khr(pdevice, surface)
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
            let surface_capabilities = surface_loader
                .get_physical_device_surface_capabilities_khr(pdevice, surface)
                .unwrap();
            let mut desired_image_count = surface_capabilities.min_image_count + 1;
            if surface_capabilities.max_image_count > 0 &&
                desired_image_count > surface_capabilities.max_image_count
            {
                desired_image_count = surface_capabilities.max_image_count;
            }

            let surface_resolution = match surface_capabilities.current_extent.width {
                std::u32::MAX => {
                    vk::Extent2D {
                        width: dimensions[0],
                        height: dimensions[1],
                    }
                }
                _ => surface_capabilities.current_extent,
            };

            let pre_transform = if surface_capabilities.supported_transforms.subset(
                vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR,
            )
            {
                vk::SURFACE_TRANSFORM_IDENTITY_BIT_KHR
            } else {
                surface_capabilities.current_transform
            };
            let present_modes = surface_loader
                .get_physical_device_surface_present_modes_khr(pdevice, surface)
                .unwrap();
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::Mailbox)
                .unwrap_or(vk::PresentModeKHR::Fifo);
            let swapchain_loader =
                Swapchain::new(&instance, &device).expect("Unable to load swapchain");
            let swapchain_create_info = vk::SwapchainCreateInfoKHR {
                s_type: vk::StructureType::SwapchainCreateInfoKhr,
                p_next: ptr::null(),
                flags: Default::default(),
                surface: surface,
                min_image_count: desired_image_count,
                image_color_space: surface_format.color_space,
                image_format: surface_format.format,
                image_extent: surface_resolution.clone(),
                image_usage: vk::IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
                image_sharing_mode: vk::SharingMode::Exclusive,
                pre_transform: pre_transform,
                composite_alpha: vk::COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
                present_mode: present_mode,
                clipped: 1,
                old_swapchain: vk::SwapchainKHR::null(),
                image_array_layers: 1,
                p_queue_family_indices: ptr::null(),
                queue_family_index_count: 0,
            };
            let swapchain = swapchain_loader
                .create_swapchain_khr(&swapchain_create_info, None)
                .unwrap();

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
                    device.create_image_view(&create_view_info, None).unwrap()
                })
                .collect();

            let device_memory_properties = instance.get_physical_device_memory_properties(pdevice);

            RenderState {
                instance: instance,
                device: device,
                pdevice: pdevice,
                queue_family_index: queue_family_index,
                device_memory_properties: device_memory_properties,
                // window stuff
                event_loop: events_loop,
                window: window,
                surface: surface,
                // swapchain
                swapchain: swapchain,
                present_images: present_images,
                present_image_views: present_image_views,
            }
        }
    }
}
