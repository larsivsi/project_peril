#[macro_use]
extern crate ash;
extern crate cgmath;
extern crate image;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate winit;

mod config;
mod nurbs;
mod object;
mod renderer;
mod scene;

use ash::util::Align;
use ash::version::DeviceV1_0;
use ash::vk;
use cgmath::{Deg, Matrix4, Point3, Rad, Vector2};
use config::Config;
use nurbs::{NURBSpline, Order};
use object::{Camera, Position};
use renderer::{MainPass, PresentPass, RenderState};
use scene::Scene;
use std::mem::{align_of, size_of};
use std::time::{Duration, SystemTime};

fn main()
{
	// init stuff
	let options_file = "options.json";
	let cfg = match Config::read_config(options_file)
	{
		Ok(cfg) => cfg,
		Err(e) =>
		{
			println!("ERROR! reading config file ({}): {}", options_file, e);
			return;
		}
	};

	let mut renderstate = RenderState::init(&cfg);
	let mut presentpass = PresentPass::init(&renderstate);
	let mut mainpass = MainPass::init(&renderstate, &cfg);
	let mut scene = Scene::new(&renderstate, &mainpass);
	let mut camera = Camera::new(Point3::new(0.0, 0.0, 0.0));
	let aspect_ratio = cfg.render_width as f32 / cfg.render_height as f32;
	let vertical_fov = Rad::from(Deg(cfg.horizontal_fov as f32 / aspect_ratio));
	let near = 1.0;
	let far = 1000.0;
	// Need to flip projection matrix due to the Vulkan NDC coordinates.
	// See https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/ for details.
	let glu_projection_matrix = cgmath::perspective(vertical_fov, aspect_ratio, near, far);
	let vulkan_ndc = Matrix4::new(1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0);
	let projection_matrix = vulkan_ndc * glu_projection_matrix;

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

	while u < spline.eval_limit()
	{
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

	let mut last_mouse_position = Vector2 {
		x: 0.0 as f64,
		y: 0.0 as f64,
	};
	let mouse_sensitivity = cfg.mouse_sensitivity;

	let move_sensitivity = 0.05;

	let w_scan_code: u32 = 17;
	let a_scan_code: u32 = 30;
	let s_scan_code: u32 = 31;
	let d_scan_code: u32 = 32;

	let up_scan_code: u32 = 103;
	let left_scan_code: u32 = 105;
	let down_scan_code: u32 = 108;
	let right_scan_code: u32 = 106;

	let esc_scan_code: u32 = 1;
	let shift_scan_code: u32 = 42;

	let mut w_down = false;
	let mut a_down = false;
	let mut s_down = false;
	let mut d_down = false;
	let mut shift_down = false;

	while running
	{
		let new_time = SystemTime::now();
		let frame_time = new_time.duration_since(current_time).expect("duration_since failed :(");
		current_time = new_time;
		accumulator += frame_time;

		while accumulator >= delta_time
		{
			scene.update();
			// animation, physics engine, scene progression etc. goes here
			accumulator -= delta_time;
			elapsed_time += delta_time;
		}

		// Update the view matrix uniform buffer
		let view_matrix = camera.generate_view_matrix();
		let view_matrix_buf_size = size_of::<Matrix4<f32>>() as u64;
		unsafe {
			let mem_ptr = renderstate
				.device
				.map_memory(mainpass.view_matrix_ub_mem, 0, view_matrix_buf_size, vk::MemoryMapFlags::empty())
				.expect("Failed to view matrix uniform memory");
			let mut mem_align = Align::new(mem_ptr, align_of::<Matrix4<f32>>() as u64, view_matrix_buf_size);
			mem_align.copy_from_slice(&[view_matrix]);
			renderstate.device.unmap_memory(mainpass.view_matrix_ub_mem);
		}

		// Do the main rendering
		let main_cmd_buf = mainpass.begin_frame(&renderstate);
		scene.draw(main_cmd_buf, mainpass.pipeline_layout, &view_matrix, &projection_matrix);
		mainpass.end_frame(&renderstate);

		// Present the rendered image
		presentpass.present_image(&renderstate, &mut mainpass.render_image);
		framecount += 1;

		if framecount % 100 == 0
		{
			// let frame_time_ms = frame_time.subsec_nanos() as f64 / 1_000_000.0;
			// println!(
			//    "frametime: {}ms => {} FPS",
			//    frame_time_ms,
			//    1_000.0 / frame_time_ms
			// );
		}

		renderstate.event_loop.poll_events(|ev| match ev
		{
			winit::Event::WindowEvent {
				event,
				..
			} => match event
			{
				winit::WindowEvent::Closed => running = false,
				// Keyboard events
				winit::WindowEvent::KeyboardInput {
					input,
					..
				} => match input.state
				{
					winit::ElementState::Pressed => match input.scancode
					{
						c if (w_scan_code == c) =>
						{
							w_down = true;
						}
						c if (a_scan_code == c) =>
						{
							a_down = true;
						}
						c if (s_scan_code == c) =>
						{
							s_down = true;
						}
						c if (d_scan_code == c) =>
						{
							d_down = true;
						}
						c if (up_scan_code == c) =>
						{
							camera.pitch(5.0);
						}
						c if (left_scan_code == c) =>
						{
							camera.yaw(-5.0);
						}
						c if (down_scan_code == c) =>
						{
							camera.pitch(-5.0);
						}
						c if (right_scan_code == c) =>
						{
							camera.yaw(5.0);
						}
						c if (esc_scan_code == c) =>
						{
							running = false;
						}
						c if (shift_scan_code == c) =>
						{
							shift_down = true;
						}
						_ =>
						{
							println!("Pressed {}", input.scancode);
						}
					},
					winit::ElementState::Released => match input.scancode
					{
						c if (w_scan_code == c) =>
						{
							w_down = false;
						}
						c if (a_scan_code == c) =>
						{
							a_down = false;
						}
						c if (s_scan_code == c) =>
						{
							s_down = false;
						}
						c if (d_scan_code == c) =>
						{
							d_down = false;
						}
						c if (shift_scan_code == c) =>
						{
							shift_down = false;
						}
						_ => (),
					},
				},
				// Mouse presses
				winit::WindowEvent::MouseInput {
					button,
					..
				} => match button
				{
					winit::MouseButton::Left =>
					{
						println!("Left mouse!");
					}
					winit::MouseButton::Right =>
					{
						println!("Right mouse!");
					}
					_ => (),
				},
				_ => (),
			},

			winit::Event::DeviceEvent {
				event,
				..
			} => match event
			{
				// Mouse Movement
				// Use DeviceEvent as it gives raw unfiltered physical motion
				winit::DeviceEvent::MouseMotion {
					delta,
					..
				} =>
				{
					println!("Mouse moved x: {} y: {}", delta.0, delta.1);
					let mut dir_change = Vector2 {
						x: (last_mouse_position.x + delta.0) * cfg.mouse_invert_x,
						y: (last_mouse_position.y + delta.1) * cfg.mouse_invert_y,
					};
					last_mouse_position.x = delta.0;
					last_mouse_position.y = delta.1;

					// Update camera.
					dir_change *= mouse_sensitivity;
					camera.yaw(dir_change.x as f32);
					camera.pitch(-dir_change.y as f32);
				}
				_ => (),
			},
			_ => (),
		});
		// Update Input.
		let mut move_speed = move_sensitivity;
		if shift_down
		{
			move_speed *= 10.0;
		}
		if w_down
		{
			let translation = camera.get_front_vector();
			camera.translate(translation * move_speed);
		}
		if a_down
		{
			let translation = camera.get_right_vector() * -1.0;
			camera.translate(translation * move_speed);
		}
		if s_down
		{
			let translation = camera.get_front_vector() * -1.0;
			camera.translate(translation * move_speed);
		}
		if d_down
		{
			let translation = camera.get_right_vector();
			camera.translate(translation * move_speed);
		}
	}

	// cleanup
}
