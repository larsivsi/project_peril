use config::Config;
use vulkano;
use vulkano::device::{Device, Queue};
use vulkano::image::SwapchainImage;
use vulkano::instance::Instance;
use vulkano::swapchain::{PresentMode, SurfaceTransform, Swapchain};
use winit;
use winit::EventsLoop;
use vulkano_win;
use vulkano_win::{Window, VkSurfaceBuild};

use std::sync::Arc;

pub struct RenderState {
    // vulkan stuff
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    // window stuff
    pub event_loop: EventsLoop,
    window: Window,
    // swapchain
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<SwapchainImage>>,
}

impl RenderState {
    pub fn init(cfg: Config) -> RenderState {
        let instance = {
            let extensions = vulkano_win::required_extensions();
            Instance::new(None, &extensions, None).expect("Failed to create Vulkan Instance")
        };

        let physical = vulkano::instance::PhysicalDevice::enumerate(&instance)
            .next()
            .expect("Failed to create device");
        println!(
            "Using device: {} (type: {:?})",
            physical.name(),
            physical.ty()
        );

        let dimensions = {
            let (width, height) = cfg.window_dimensions;
            [width, height]
        };

        let events_loop = winit::EventsLoop::new();
        let window = winit::WindowBuilder::new()
            .with_dimensions(dimensions[0], dimensions[1])
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();

        let queue = physical
            .queue_families()
            .find(|&q| {
                q.supports_graphics() && window.surface().is_supported(q).unwrap_or(false)
            })
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            let device_ext = vulkano::device::DeviceExtensions {
                khr_swapchain: true,
                ..vulkano::device::DeviceExtensions::none()
            };

            Device::new(
                physical,
                physical.supported_features(),
                &device_ext,
                [(queue, 0.5)].iter().cloned(),
            ).expect("failed to create device")
        };

        let queue = queues.next().unwrap();

        let (mut swapchain, mut images) = {
            let caps = window
                .surface()
                .capabilities(device.physical_device())
                .expect("failed to get surface capabilities");
            let alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            Swapchain::new(
                device.clone(),
                window.surface().clone(),
                caps.min_image_count,
                format,
                caps.current_extent.unwrap(),
                1,
                caps.supported_usage_flags,
                &queue,
                SurfaceTransform::Identity,
                alpha,
                PresentMode::Fifo,
                true,
                None,
            ).expect("failed to create swapchain")
        };

        RenderState {
            instance: instance.clone(),
            device: device,
            queue: queue,

            event_loop: events_loop,
            window: window,

            swapchain: swapchain,
            images: images,
        }
    }
}
