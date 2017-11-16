use config::Config;
use vulkano;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::Instance;
use vulkano::swapchain;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain};
use winit;
use winit::EventsLoop;
use vulkano_win;
use vulkano_win::{Window, VkSurfaceBuild};

use std::mem;
use std::sync::Arc;

pub struct RenderState {
    // vulkan stuff
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    renderpass: Arc<RenderPassAbstract>,
    // window stuff
    pub event_loop: EventsLoop,
    window: Window,
    // swapchain
    swapchain: Arc<Swapchain>,
    output_images: Vec<Arc<SwapchainImage>>,
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

        let (swapchain, images) = {
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

        let renderpass = Arc::new(
            single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.format(),
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
).unwrap(),
        );

        RenderState {
            instance: instance.clone(),
            device: device,
            queue: queue,
            renderpass: renderpass,

            event_loop: events_loop,
            window: window,

            swapchain: swapchain,
            output_images: images,
        }
    }

    // In case window changes dimensions (resized etc.)
    pub fn recreate_swapchain(&mut self) {
        let dimensions = {
            let (width, height) = self.window.window().get_inner_size_pixels().unwrap();
            [width, height]
        };

        let (swapchain, images) = match self.swapchain.recreate_with_dimension(dimensions) {
            Ok(r) => r,
            Err(err) => panic!("{:?}", err),
        };

        mem::replace(&mut self.swapchain, swapchain);
        mem::replace(&mut self.output_images, images);
    }
}

/* TODO: when used, remove the dead code allowence */
#[allow(dead_code)]
pub struct Frame {
    framebuffer: Arc<FramebufferAbstract>,
    commandbuffer: AutoCommandBufferBuilder,
}

impl Frame {
    pub fn new(renderstate: &RenderState) -> Frame {
        let (output_idx, _acquire_future) =
            match swapchain::acquire_next_image(renderstate.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => panic!("out of date :("),
                Err(err) => panic!("{:?}", err),
            };

        let framebuffer = Arc::new(
            Framebuffer::start(renderstate.renderpass.clone())
                .add(renderstate.output_images[output_idx].clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let commandbuffer = AutoCommandBufferBuilder::primary_one_time_submit(
            renderstate.device.clone(),
            renderstate.queue.as_ref().family(),
        ).unwrap();

        Frame {
            framebuffer: framebuffer,
            commandbuffer: commandbuffer,
        }

    }
}
