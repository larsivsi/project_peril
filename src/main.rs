#[macro_use]
extern crate ash;
extern crate bit_vec;
extern crate cgmath;
extern crate image;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate winit;

mod config;
mod input;
mod nurbs;
mod object;
mod renderer;
mod scene;

use ash::util::Align;
use ash::version::DeviceV1_0;
use ash::vk;
use cgmath::{Deg, Matrix4, Point3, Rad, Vector2, Vector3};
use config::Config;
use input::InputState;
use input::KeyIndex;
use nurbs::{NURBSpline, Order};
use object::{Camera, Position};
use renderer::{MainPass, PresentPass, RenderState};
use scene::Scene;
use std::io::Write;
use std::mem::{align_of, size_of};
use std::time::{Duration, SystemTime};

const ENGINE_TARGET_HZ: u64 = 60;
const ENGINE_TIMESTEP: Duration = Duration::from_nanos(1_000_000_000 / ENGINE_TARGET_HZ);

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
	let mut frames_per_second: u32 = 0;
	let mut second_accumulator = Duration::new(0, 0);
	let mut engine_accumulator = Duration::new(0, 0);
	let mut last_timestamp = SystemTime::now();

	let mut last_mouse_position = Vector2 {
		x: 0.0 as f64,
		y: 0.0 as f64,
	};
	let mouse_sensitivity = cfg.mouse_sensitivity;
	let move_sensitivity = 0.3;

	let mut input_state = InputState::new();
	let mut cursor_captured = false;
	let mut cursor_dirty = true;

	while running
	{
		let current_timestamp = SystemTime::now();
		let frame_time = current_timestamp.duration_since(last_timestamp).unwrap();
		last_timestamp = current_timestamp;
		engine_accumulator += frame_time;
		second_accumulator += frame_time;

		// ENGINE
		//   Fixed engine timestep
		while engine_accumulator >= ENGINE_TIMESTEP
		{
			// Update Input.
			let mut move_speed = move_sensitivity;
			if input_state.get(KeyIndex::SPRINT)
			{
				move_speed *= 10.0;
			}
			if input_state.get(KeyIndex::FORWARD)
			{
				let translation = camera.get_cam_front();
				camera.translate(translation * move_speed);
			}
			if input_state.get(KeyIndex::LEFT)
			{
				let translation = camera.get_cam_right() * -1.0;
				camera.translate(translation * move_speed);
			}
			if input_state.get(KeyIndex::BACK)
			{
				let translation = camera.get_cam_front() * -1.0;
				camera.translate(translation * move_speed);
			}
			if input_state.get(KeyIndex::RIGHT)
			{
				let translation = camera.get_cam_right();
				camera.translate(translation * move_speed);
			}
			if input_state.get(KeyIndex::UP)
			{
				let translation = Vector3::unit_y();
				camera.translate(translation * move_speed);
			}
			if input_state.get(KeyIndex::DOWN)
			{
				let translation = Vector3::unit_y() * -1.0;
				camera.translate(translation * move_speed);
			}

			// animation, physics engine, scene progression etc. goes here
			scene.update();

			engine_accumulator -= ENGINE_TIMESTEP;
		}

		// RENDER
		//   Update the view matrix uniform buffer
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

		//   Do the main rendering
		let main_cmd_buf = mainpass.begin_frame(&renderstate);
		scene.draw(main_cmd_buf, mainpass.pipeline_layout, &view_matrix, &projection_matrix);
		mainpass.end_frame(&renderstate);

		//   Present the rendered image
		presentpass.present_image(&renderstate, &mut mainpass.render_image);

		//   Update and potentially print FPS
		frames_per_second += 1;
		if second_accumulator > Duration::from_secs(1)
		{
			let term_fps = format!("\r{} FPS", frames_per_second).into_bytes();
			std::io::stdout().write(&term_fps).unwrap();
			std::io::stdout().flush().unwrap();
			frames_per_second = 0;
			second_accumulator = Duration::new(0, 0);
		}

		// INPUT
		renderstate.event_loop.poll_events(|ev| match ev
		{
			winit::Event::WindowEvent {
				event,
				..
			} => match event
			{
				winit::WindowEvent::CloseRequested => running = false,
				winit::WindowEvent::Focused(has_focus) =>
				{
					cursor_captured = has_focus;
					cursor_dirty = true;
				}
				// Keyboard events
				winit::WindowEvent::KeyboardInput {
					input,
					..
				} =>
				{
					input_state.update_key(input);
					if input_state.get(KeyIndex::TERMINATE)
					{
						running = false;
					}
				}
				// Mouse presses
				winit::WindowEvent::MouseInput {
					button,
					..
				} =>
				{
					if cursor_captured
					{
						match button
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
						}
					}
				}
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
					if cursor_captured
					{
						// println!("Mouse moved x: {} y: {}", delta.0, delta.1);
						let mut dir_change = Vector2 {
							x: (last_mouse_position.x + delta.0),
							y: (last_mouse_position.y + delta.1),
						};
						last_mouse_position.x = delta.0;
						last_mouse_position.y = delta.1;

						// Update camera.
						dir_change *= mouse_sensitivity;
						camera.yaw(match cfg.mouse_invert_x
						{
							true => dir_change.x,
							false => -dir_change.x,
						} as f32);
						camera.pitch(match cfg.mouse_invert_y
						{
							true => dir_change.y,
							false => -dir_change.y,
						} as f32);
					}
				}
				_ => (),
			},
			_ => (),
		});

		if cursor_dirty
		{
			if cursor_captured
			{
				renderstate.window.grab_cursor(true).expect("Failed to grab pointer");
				renderstate.window.hide_cursor(true);
			}
			else
			{
				renderstate.window.grab_cursor(false).expect("Failed to return pointer");
				renderstate.window.hide_cursor(false);
			}
			cursor_dirty = false;
		}
	}

	// Cleanup terminal
	print!("\n");
}
