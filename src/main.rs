#[macro_use]
extern crate ash;
extern crate cgmath;
extern crate image;
extern crate regex;
extern crate winit;

mod config;
mod nurbs;
mod object;
mod renderer;
mod scene;

use ash::version::DeviceV1_0;
use cgmath::Point3;
use config::Config;
use nurbs::{Order, NURBSpline};
use object::Camera;
use renderer::{PresentState, RenderState};
use scene::Scene;
use std::time::{Duration, SystemTime};

fn main() {
    // init stuff
    let cfg = Config::read_config("options.cfg");

    let mut renderstate = RenderState::init(&cfg);
    let mut presentstate = PresentState::init(&renderstate);
    let _scene = Scene::new(&renderstate);
    let camera = Camera::new(Point3::new(0.0, 0.0, 0.0));
    let _view_matrix = camera.generate_view_matrix();

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
        let _point = spline.evaluate_at(u);
        u += step;
    }

    // main loop
    let mut running = true;
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

        //call to render function goes here
        let cmd_buf;
        let res = presentstate.begin_frame(&renderstate);
        match res {
            Some(buf) => {
                cmd_buf = buf;
            }
            None => {
                // Swapchain was outdated, but now one was created.
                // Skip this frame.
                continue;
            }
        }
        // Draw stuff
        unsafe {
            // just fake six vertices for now
            renderstate.device.cmd_draw(cmd_buf, 6, 1, 0, 0);
        }
        //then swapbuffers etc.
        presentstate.end_frame_and_present(&renderstate);
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
            _ => (),
        });

    }

    //cleanup
}
