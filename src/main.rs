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
use cgmath::{Deg, Matrix4, Point3, Rad, Vector3};
use config::Config;
use input::{Action, InputState};
use nurbs::{NURBSpline, Order};
use object::transform::Transformable;
use object::Camera;
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

	let mut input_state = InputState::new();
	let mut cursor_captured = false;
	let mut cursor_state_dirty = true;

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
			if input_state.has_actions()
			{
				let mut move_speed = 0.3;
				if input_state.action_requested(Action::SPRINT)
				{
					move_speed *= 10.0;
				}
				if input_state.action_requested(Action::FORWARD)
				{
					let translation = camera.get_front_vector();
					camera.translate(translation * move_speed);
				}
				if input_state.action_requested(Action::LEFT)
				{
					let translation = camera.get_right_vector() * -1.0;
					camera.translate(translation * move_speed);
				}
				if input_state.action_requested(Action::BACK)
				{
					let translation = camera.get_front_vector() * -1.0;
					camera.translate(translation * move_speed);
				}
				if input_state.action_requested(Action::RIGHT)
				{
					let translation = camera.get_right_vector();
					camera.translate(translation * move_speed);
				}
				if input_state.action_requested(Action::UP)
				{
					let translation = Vector3::unit_y();
					camera.translate(translation * move_speed);
				}
				if input_state.action_requested(Action::DOWN)
				{
					let translation = Vector3::unit_y() * -1.0;
					camera.translate(translation * move_speed);
				}
				if input_state.action_requested(Action::CAM_UP)
				{
					camera.pitch(5.0);
				}
				if input_state.action_requested(Action::CAM_LEFT)
				{
					camera.yaw(5.0);
				}
				if input_state.action_requested(Action::CAM_DOWN)
				{
					camera.pitch(-5.0);
				}
				if input_state.action_requested(Action::CAM_RIGHT)
				{
					camera.yaw(-5.0);
				}
			}

			let (mut mouse_yaw, mut mouse_pitch) = input_state.get_and_clear_mouse_delta();
			if cursor_captured && (mouse_yaw != 0.0 || mouse_pitch != 0.0)
			{
				// Yaw and pitch will be in the opposite direction of mouse delta
				mouse_yaw *= if cfg.mouse_invert_x
				{
					cfg.mouse_sensitivity
				}
				else
				{
					-cfg.mouse_sensitivity
				};
				mouse_pitch *= if cfg.mouse_invert_y
				{
					cfg.mouse_sensitivity
				}
				else
				{
					-cfg.mouse_sensitivity
				};

				camera.yaw(mouse_yaw as f32);
				camera.pitch(mouse_pitch as f32);
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
		scene.draw(&renderstate.device, main_cmd_buf, mainpass.pipeline_layout, &view_matrix, &projection_matrix);
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
					cursor_state_dirty = true;
				}
				// Keyboard events
				winit::WindowEvent::KeyboardInput {
					input,
					..
				} =>
				{
					input_state.update_key(input);
					// Handle some state that should not wait for another frame
					if input_state.action_requested(Action::TERMINATE)
					{
						running = false;
					}
					if input_state.action_requested(Action::CURSOR_CAPTURE_TOGGLE)
					{
						cursor_captured = !cursor_captured;
						cursor_state_dirty = true;
					}
				}
				// Mouse presses
				winit::WindowEvent::MouseInput {
					button,
					state,
					..
				} => input_state.update_mouse_button(button, state),
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
				} => input_state.update_mouse_movement(delta),
				_ => (),
			},
			_ => (),
		});

		if cursor_state_dirty
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
			cursor_state_dirty = false;
		}
	}

	// Cleanup terminal
	print!("\n");
}
