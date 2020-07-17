use crate::core::{InputConsumer, MouseConsumer};
use bit_vec::BitVec;
use std::cell::RefCell;
use std::rc::Rc;
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

pub enum ActionType
{
	IMMEDIATE,
	TICK,
}

struct Consumer
{
	actions: BitVec,
	ptr: Rc<RefCell<dyn InputConsumer>>,
}

pub struct InputState
{
	actions: BitVec,
	mouse_delta: (f64, f64),
}

pub struct InputHandler
{
	state: InputState,
	last_mouse_pos: (f64, f64),
	tick_action_consumers: Vec<Consumer>,
	immediate_action_consumers: Vec<Consumer>,
	mouse_consumer: Option<Rc<RefCell<dyn MouseConsumer>>>,
}

impl InputHandler
{
	pub fn new() -> InputHandler
	{
		InputHandler {
			state: InputState {
				actions: BitVec::from_elem(Action::LENGTH_OF_ENUM as usize, false),
				mouse_delta: (0.0, 0.0),
			},
			last_mouse_pos: (0.0, 0.0),
			// Can at most have LENGTH_OF_ENUM different consumers
			tick_action_consumers: Vec::with_capacity(Action::LENGTH_OF_ENUM as usize),
			immediate_action_consumers: Vec::with_capacity(Action::LENGTH_OF_ENUM as usize),
			mouse_consumer: None,
		}
	}

	pub fn update_key(&mut self, event: KeyboardInput)
	{
		match event.scancode
		{
			W_SCAN_CODE => self.state.actions.set(Action::FORWARD as usize, event.state == ElementState::Pressed),
			A_SCAN_CODE => self.state.actions.set(Action::LEFT as usize, event.state == ElementState::Pressed),
			S_SCAN_CODE => self.state.actions.set(Action::BACK as usize, event.state == ElementState::Pressed),
			D_SCAN_CODE => self.state.actions.set(Action::RIGHT as usize, event.state == ElementState::Pressed),
			SPACE_SCAN_CODE => self.state.actions.set(Action::UP as usize, event.state == ElementState::Pressed),
			LCTRL_SCAN_CODE => self.state.actions.set(Action::DOWN as usize, event.state == ElementState::Pressed),
			LSHIFT_SCAN_CODE => self.state.actions.set(Action::SPRINT as usize, event.state == ElementState::Pressed),
			UP_SCAN_CODE => self.state.actions.set(Action::CAM_UP as usize, event.state == ElementState::Pressed),
			LEFT_SCAN_CODE => self.state.actions.set(Action::CAM_LEFT as usize, event.state == ElementState::Pressed),
			DOWN_SCAN_CODE => self.state.actions.set(Action::CAM_DOWN as usize, event.state == ElementState::Pressed),
			RIGHT_SCAN_CODE => self.state.actions.set(Action::CAM_RIGHT as usize, event.state == ElementState::Pressed),
			ESC_SCAN_CODE => self.state.actions.set(Action::TERMINATE as usize, event.state == ElementState::Pressed),
			F_SCAN_CODE =>
			{
				self.state.actions.set(Action::CURSOR_CAPTURE_TOGGLE as usize, event.state == ElementState::Pressed)
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
				println!("Unmapped key {} ({:?}) {}", event.scancode, event.virtual_keycode.unwrap(), statestr);
			}
		}

		// Early out if there's nothing to do
		if self.state.actions.none()
		{
			return;
		}

		// Handle immediate consumers
		for consumer in self.immediate_action_consumers.iter()
		{
			let mut intersection = self.state.actions.clone();
			intersection.and(&consumer.actions);
			if intersection.any()
			{
				consumer.ptr.borrow_mut().consume(intersection);
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

	pub fn register_actions<T: InputConsumer + 'static>(
		&mut self, actions_consumed: BitVec, action_type: ActionType, consumer: Rc<RefCell<T>>,
	)
	{
		debug_assert_eq!(actions_consumed.len(), Action::LENGTH_OF_ENUM as usize);

		// Cannot register same action twice
		if cfg!(debug_assertions)
		{
			for consumer in self.immediate_action_consumers.iter()
			{
				let mut intersection = actions_consumed.clone();
				intersection.and(&consumer.actions);
				debug_assert!(intersection.none());
			}

			for consumer in self.tick_action_consumers.iter()
			{
				let mut intersection = actions_consumed.clone();
				intersection.and(&consumer.actions);
				debug_assert!(intersection.none());
			}
		}

		match action_type
		{
			ActionType::IMMEDIATE => self.immediate_action_consumers.push(Consumer {
				actions: actions_consumed,
				ptr: consumer,
			}),
			ActionType::TICK => self.tick_action_consumers.push(Consumer {
				actions: actions_consumed,
				ptr: consumer,
			}),
		}
	}

	pub fn actions_tick(&self)
	{
		// Early out if there's nothing to do
		if self.state.actions.none()
		{
			return;
		}

		for consumer in self.tick_action_consumers.iter()
		{
			let mut intersection = self.state.actions.clone();
			intersection.and(&consumer.actions);
			if intersection.any()
			{
				consumer.ptr.borrow_mut().consume(intersection);
			}
		}
	}

	pub fn register_mouse_movement<T: MouseConsumer + 'static>(
		&mut self, consumer: Rc<RefCell<T>>, mouse_invert: (bool, bool), mouse_sensitivity: f64,
	)
	{
		consumer.borrow_mut().register_mouse_settings(mouse_invert, mouse_sensitivity);
		self.mouse_consumer = Some(consumer);
	}

	pub fn update_mouse_movement(&mut self, mouse_delta: (f64, f64))
	{
		let change = (self.last_mouse_pos.0 + mouse_delta.0, self.last_mouse_pos.1 + mouse_delta.1);
		self.last_mouse_pos.0 = mouse_delta.0;
		self.last_mouse_pos.1 = mouse_delta.1;
		self.state.mouse_delta.0 += change.0;
		self.state.mouse_delta.1 += change.1;
	}

	pub fn mouse_movement_tick(&mut self, cursor_captured: bool)
	{
		if self.state.mouse_delta == (0.0, 0.0)
		{
			return;
		}

		if cursor_captured
		{
			match &self.mouse_consumer
			{
				Some(consumer) =>
				{
					consumer.borrow_mut().consume(self.state.mouse_delta);
				}
				None => (),
			}
		}

		self.state.mouse_delta = (0.0, 0.0);
	}
}
