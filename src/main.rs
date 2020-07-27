mod core;
mod game;
mod renderer;

use crate::core::{Action, ActionType, Config, InputConsumer, InputHandler, KeyEventState};
use crate::game::Scene;
use crate::renderer::{MainPass, PresentPass, RenderState};
use ash::util::Align;
use ash::version::DeviceV1_0;
use ash::vk;
use bit_vec::BitVec;
use cgmath::{Deg, Matrix4, Rad};
use sdl2::event::{Event, WindowEvent};
use std::cell::RefCell;
use std::io::Write;
use std::mem::{align_of, size_of};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

const ENGINE_TARGET_HZ: u64 = 60;
const ENGINE_TIMESTEP: Duration = Duration::from_nanos(1_000_000_000 / ENGINE_TARGET_HZ);

struct EngineState
{
	pub running: bool,
	pub cursor_captured: bool,
	pub cursor_state_dirty: bool,
}

impl EngineState
{
	fn new() -> EngineState
	{
		return EngineState {
			running: true,
			cursor_captured: false,
			cursor_state_dirty: true,
		};
	}
}

impl InputConsumer for EngineState
{
	fn get_handled_actions(&self) -> BitVec
	{
		let mut handled_actions = BitVec::from_elem(Action::LENGTH_OF_ENUM as usize, false);

		handled_actions.set(Action::TERMINATE as usize, true);
		handled_actions.set(Action::CURSOR_CAPTURE_TOGGLE as usize, true);

		return handled_actions;
	}
	fn consume(&mut self, actions: BitVec)
	{
		if actions.get(Action::TERMINATE as usize).unwrap()
		{
			self.running = false;
		}
		if actions.get(Action::CURSOR_CAPTURE_TOGGLE as usize).unwrap()
		{
			self.cursor_captured = !self.cursor_captured;
			self.cursor_state_dirty = true;
		}
	}
}

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

	let sdl_context = sdl2::init().unwrap();
	let video_subsystem = sdl_context.video().unwrap();
	let renderstate = RenderState::init(&cfg, &video_subsystem);
	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut presentpass = PresentPass::init(&renderstate);
	let mut loading_image = renderstate.load_image("assets/original/textures/project_peril_logo.png", true);
	presentpass.present_image(&renderstate, &mut loading_image);
	let mut mainpass = MainPass::init(&renderstate, &cfg);
	let mut input_handler = InputHandler::new();
	let engine_state = Rc::new(RefCell::new(EngineState::new()));
	input_handler.register_actions(
		engine_state.borrow().get_handled_actions(),
		ActionType::IMMEDIATE,
		engine_state.clone(),
	);
	let mut scene = Scene::new(&renderstate, &mainpass, &cfg, &mut input_handler);
	let aspect_ratio = cfg.render_width as f32 / cfg.render_height as f32;
	let vertical_fov = Rad::from(Deg(cfg.horizontal_fov as f32 / aspect_ratio));
	let near = 1.0;
	let far = 1000.0;
	// Need to flip projection matrix due to the Vulkan NDC coordinates.
	// See https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/ for details.
	let glu_projection_matrix = cgmath::perspective(vertical_fov, aspect_ratio, near, far);
	let vulkan_ndc = Matrix4::new(1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0);
	let projection_matrix = vulkan_ndc * glu_projection_matrix;

	// main loop
	let mut frames_per_second: u32 = 0;
	let mut second_accumulator = Duration::new(0, 0);
	let mut engine_accumulator = Duration::new(0, 0);
	let mut last_timestamp = SystemTime::now();

	while engine_state.borrow().running
	{
		let current_timestamp = SystemTime::now();
		let frame_time = current_timestamp.duration_since(last_timestamp).unwrap();
		last_timestamp = current_timestamp;
		engine_accumulator += frame_time;
		second_accumulator += frame_time;

		// ENGINE
		//   Mouse movement ticks once per frame
		input_handler.mouse_movement_tick(engine_state.borrow().cursor_captured);
		//   Fixed engine timestep
		while engine_accumulator >= ENGINE_TIMESTEP
		{
			// Actions tick once per timestep.
			input_handler.actions_tick();

			// animation, physics engine, scene progression etc. goes here
			scene.update();

			engine_accumulator -= ENGINE_TIMESTEP;
		}

		// RENDER
		//   Update the view matrix uniform buffer
		let view_matrix = scene.get_view_matrix();
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
		for event in event_pump.poll_iter()
		{
			match event
			{
				Event::Quit {
					..
				} => engine_state.borrow_mut().running = false,
				Event::KeyDown {
					scancode,
					..
				} => input_handler.update_key(scancode.unwrap(), KeyEventState::PRESSED),
				Event::KeyUp {
					scancode,
					..
				} => input_handler.update_key(scancode.unwrap(), KeyEventState::RELEASED),
				Event::MouseButtonDown {
					mouse_btn,
					..
				} => input_handler.update_mouse_button(mouse_btn, KeyEventState::PRESSED),
				Event::MouseButtonUp {
					mouse_btn,
					..
				} => input_handler.update_mouse_button(mouse_btn, KeyEventState::RELEASED),
				Event::MouseMotion {
					xrel,
					yrel,
					..
				} => input_handler.update_mouse_movement((xrel, yrel)),
				Event::Window {
					win_event,
					..
				} => match win_event
				{
					WindowEvent::FocusGained =>
					{
						engine_state.borrow_mut().cursor_captured = true;
						engine_state.borrow_mut().cursor_state_dirty = true;
					}
					WindowEvent::FocusLost =>
					{
						engine_state.borrow_mut().cursor_captured = false;
						engine_state.borrow_mut().cursor_state_dirty = true;
					}
					_ =>
					{}
				},
				_ =>
				{}
			}
		}

		if engine_state.borrow().cursor_state_dirty
		{
			if engine_state.borrow().cursor_captured
			{
				sdl_context.mouse().set_relative_mouse_mode(true);
			}
			else
			{
				sdl_context.mouse().set_relative_mouse_mode(false);
			}
			engine_state.borrow_mut().cursor_state_dirty = false;
		}
	}

	// Cleanup
	loading_image.destroy(&renderstate.device);
	print!("\n");
}
