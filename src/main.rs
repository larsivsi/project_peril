#[macro_use]
extern crate ash;
extern crate cgmath;
extern crate regex;
extern crate winit;

mod config;
mod nurbs;
mod object;
mod renderer;
mod scene;

use ash::vk;
use cgmath::Point3;
use config::Config;
use nurbs::{Order, NURBSpline};
use object::Camera;
use renderer::{CommandBuffers, Pipeline, RenderState};
use scene::Scene;
use std::ptr;
use std::time::{Duration, SystemTime};

fn main() {
    // init stuff
    let cfg = Config::read_config("options.cfg");

    let mut renderstate = RenderState::init(cfg);
    let pipeline = Pipeline::new(&renderstate);
    let cmd_buffers = CommandBuffers::new(&renderstate, &pipeline);
    let scene = Scene::new(&renderstate);
    let _camera = Camera::new(Point3::new(0.0, 0.0, 0.0));

    let points = vec![
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
        Point3::new(-1.0, 0.0, 0.0),
        Point3::new(0.0, -1.0, 0.0),
        Point3::new(0.0, 0.0, 1.0),
        Point3::new(0.0, 0.0, -1.0),
        Point3::new(0.0, 1.0, -1.0),
        Point3::new(1.0, 0.0, -1.0),
    ];

    let mut u = 0.0;
    let step = 0.1;
    let spline = NURBSpline::new(Order::CUBIC, points);

    while u < spline.eval_limit() {
        let point = spline.evaluate_at(u);
        u += step;
    }

    // main loop
    let mut running = true;
    let mut recreate_swapchain = false;
    let mut framecount: u64 = 0;
    // aim for 60fps = 16.66666... ms
    let delta_time = Duration::from_millis(17);
    let mut elapsed_time = Duration::new(0, 0);
    let mut accumulator = Duration::new(0, 0);
    let mut current_time = SystemTime::now();

    while running {
        let new_time = SystemTime::now();
        let frame_time = new_time.duration_since(current_time).expect(
            "duration_since failed :(",
        );
        current_time = new_time;
        accumulator += frame_time;

        while accumulator >= delta_time {
            //animation, physics engine, scene progression etc. goes here
            accumulator -= delta_time;
            elapsed_time += delta_time;
        }

        if recreate_swapchain {
            renderstate.recreate_swapchain();
            recreate_swapchain = false;
        }

        //call to render function goes here
        let present_idx;
        unsafe {
            present_idx = renderstate
                .swapchain_loader
                .acquire_next_image_khr(
                    renderstate.swapchain,
                    std::u64::MAX,
                    renderstate.image_available_sem,
                    vk::Fence::null(),
                )
                .unwrap();
        }
        // Draw stuff:
        renderer::draw(&renderstate, &pipeline, &cmd_buffers, present_idx as usize);
        //then swapbuffers etc.
        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PresentInfoKhr,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &renderstate.rendering_finished_sem,
            swapchain_count: 1,
            p_swapchains: &renderstate.swapchain,
            p_image_indices: &present_idx,
            p_results: ptr::null_mut(),
        };
        unsafe {
            renderstate
                .swapchain_loader
                .queue_present_khr(renderstate.present_queue, &present_info)
                .unwrap();
        }
        framecount += 1;

        if framecount % 100 == 0 {
            let frame_time_ms = frame_time.subsec_nanos() as f64 / 1_000_000.0;
            println!(
                "frametime: {}ms => {} FPS",
                frame_time_ms,
                1_000.0 / frame_time_ms
            );
        }

        renderstate.event_loop.poll_events(|ev| match ev {
            winit::Event::WindowEvent { event: winit::WindowEvent::Closed, .. } => running = false,
            winit::Event::WindowEvent { event: winit::WindowEvent::Resized(_, _), .. } => {
                recreate_swapchain = true
            }
            _ => (),
        });

    }

    //cleanup
}
