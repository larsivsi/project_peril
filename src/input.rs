use bit_vec::BitVec;
use winit::{ElementState, KeyboardInput, MouseButton};

const W_SCAN_CODE: u32 = 17;
const A_SCAN_CODE: u32 = 30;
const S_SCAN_CODE: u32 = 31;
const D_SCAN_CODE: u32 = 32;
const F_SCAN_CODE: u32 = 33;

const UP_SCAN_CODE: u32 = 103;
const LEFT_SCAN_CODE: u32 = 105;
const DOWN_SCAN_CODE: u32 = 108;
const RIGHT_SCAN_CODE: u32 = 106;

const ESC_SCAN_CODE: u32 = 1;
const SPACE_SCAN_CODE: u32 = 57;
const LSHIFT_SCAN_CODE: u32 = 42;
const LCTRL_SCAN_CODE: u32 = 29;

#[allow(non_camel_case_types)]
pub enum Action
{
	FORWARD,
	BACK,
	LEFT,
	RIGHT,
	UP,
	DOWN,
	SPRINT,
	CAM_UP,
	CAM_DOWN,
	CAM_LEFT,
	CAM_RIGHT,
	CURSOR_CAPTURE_TOGGLE,
	TERMINATE,
	LENGTH_OF_ENUM,
}

pub struct InputState
{
	actions: BitVec,
	last_mouse_pos: (f64, f64),
	mouse_delta: (f64, f64),
}

impl InputState
{
	pub fn new() -> InputState
	{
		InputState {
			actions: BitVec::from_elem(Action::LENGTH_OF_ENUM as usize, false),
			last_mouse_pos: (0.0, 0.0),
			mouse_delta: (0.0, 0.0),
		}
	}

	pub fn update_key(&mut self, event: KeyboardInput)
	{
		match event.scancode
		{
			W_SCAN_CODE => self.actions.set(Action::FORWARD as usize, event.state == ElementState::Pressed),
			A_SCAN_CODE => self.actions.set(Action::LEFT as usize, event.state == ElementState::Pressed),
			S_SCAN_CODE => self.actions.set(Action::BACK as usize, event.state == ElementState::Pressed),
			D_SCAN_CODE => self.actions.set(Action::RIGHT as usize, event.state == ElementState::Pressed),
			SPACE_SCAN_CODE => self.actions.set(Action::UP as usize, event.state == ElementState::Pressed),
			LCTRL_SCAN_CODE => self.actions.set(Action::DOWN as usize, event.state == ElementState::Pressed),
			LSHIFT_SCAN_CODE => self.actions.set(Action::SPRINT as usize, event.state == ElementState::Pressed),
			UP_SCAN_CODE => self.actions.set(Action::CAM_UP as usize, event.state == ElementState::Pressed),
			LEFT_SCAN_CODE => self.actions.set(Action::CAM_LEFT as usize, event.state == ElementState::Pressed),
			DOWN_SCAN_CODE => self.actions.set(Action::CAM_DOWN as usize, event.state == ElementState::Pressed),
			RIGHT_SCAN_CODE => self.actions.set(Action::CAM_RIGHT as usize, event.state == ElementState::Pressed),
			ESC_SCAN_CODE => self.actions.set(Action::TERMINATE as usize, event.state == ElementState::Pressed),
			F_SCAN_CODE =>
			{
				self.actions.set(Action::CURSOR_CAPTURE_TOGGLE as usize, event.state == ElementState::Pressed)
			}
			_ =>
			{
				let statestr = if event.state == ElementState::Pressed
				{
					"pressed"
				}
				else
				{
					"released"
				};
				println!("Unmapped key {} {}", event.scancode, statestr);
			}
		}
	}

	pub fn update_mouse_button(&mut self, button: MouseButton, state: ElementState)
	{
		let statestr = if state == ElementState::Pressed
		{
			"pressed"
		}
		else
		{
			"released"
		};
		match button
		{
			// Currently not mapped to any actions
			winit::MouseButton::Left =>
			{
				println!("Left mouse {}!", statestr);
			}
			winit::MouseButton::Right =>
			{
				println!("Right mouse {}!", statestr);
			}
			winit::MouseButton::Middle =>
			{
				println!("Middle mouse {}!", statestr);
			}
			_ => (),
		}
	}

	pub fn update_mouse_movement(&mut self, mouse_delta: (f64, f64))
	{
		let change = (self.last_mouse_pos.0 + mouse_delta.0, self.last_mouse_pos.1 + mouse_delta.1);
		self.last_mouse_pos.0 = mouse_delta.0;
		self.last_mouse_pos.1 = mouse_delta.1;
		self.mouse_delta.0 += change.0;
		self.mouse_delta.1 += change.1;
	}

	pub fn action_requested(&self, bv_idx: Action) -> bool
	{
		return self.actions.get(bv_idx as usize).unwrap();
	}

	pub fn has_actions(&self) -> bool
	{
		return !self.actions.none();
	}

	pub fn get_and_clear_mouse_delta(&mut self) -> (f64, f64)
	{
		let current_delta = (self.mouse_delta.0, self.mouse_delta.1);
		self.mouse_delta = (0.0, 0.0);
		return current_delta;
	}
}
