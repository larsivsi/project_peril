use bit_vec::BitVec;
use sdl2::keyboard::Scancode;
use sdl2::mouse::MouseButton;
use std::cell::RefCell;
use std::rc::Rc;

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

#[derive(PartialEq)]
pub enum KeyEventState
{
	PRESSED,
	RELEASED,
}

struct Consumer
{
	actions: BitVec,
	ptr: Rc<RefCell<dyn InputConsumer>>,
}

pub trait InputConsumer
{
	fn get_handled_actions(&self) -> BitVec;
	fn consume(&mut self, actions: BitVec);
}

pub trait MouseConsumer
{
	fn register_mouse_settings(&mut self, mouse_invert: (bool, bool), mouse_sensitivity: f32);
	fn consume(&mut self, mouse_delta: (i32, i32));
}

struct InputState
{
	actions: BitVec,
	mouse_delta: (i32, i32),
}

pub struct InputHandler
{
	state: InputState,
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
				mouse_delta: (0, 0),
			},
			// Can at most have LENGTH_OF_ENUM different consumers
			tick_action_consumers: Vec::with_capacity(Action::LENGTH_OF_ENUM as usize),
			immediate_action_consumers: Vec::with_capacity(Action::LENGTH_OF_ENUM as usize),
			mouse_consumer: None,
		}
	}

	pub fn update_key(&mut self, scancode: Scancode, event_state: KeyEventState)
	{
		match scancode
		{
			Scancode::W => self.state.actions.set(Action::FORWARD as usize, event_state == KeyEventState::PRESSED),
			Scancode::A => self.state.actions.set(Action::LEFT as usize, event_state == KeyEventState::PRESSED),
			Scancode::S => self.state.actions.set(Action::BACK as usize, event_state == KeyEventState::PRESSED),
			Scancode::D => self.state.actions.set(Action::RIGHT as usize, event_state == KeyEventState::PRESSED),
			Scancode::Space => self.state.actions.set(Action::UP as usize, event_state == KeyEventState::PRESSED),
			Scancode::LCtrl => self.state.actions.set(Action::DOWN as usize, event_state == KeyEventState::PRESSED),
			Scancode::LShift => self.state.actions.set(Action::SPRINT as usize, event_state == KeyEventState::PRESSED),
			Scancode::Up => self.state.actions.set(Action::CAM_UP as usize, event_state == KeyEventState::PRESSED),
			Scancode::Left => self.state.actions.set(Action::CAM_LEFT as usize, event_state == KeyEventState::PRESSED),
			Scancode::Down => self.state.actions.set(Action::CAM_DOWN as usize, event_state == KeyEventState::PRESSED),
			Scancode::Right =>
			{
				self.state.actions.set(Action::CAM_RIGHT as usize, event_state == KeyEventState::PRESSED)
			}
			Scancode::Escape =>
			{
				self.state.actions.set(Action::TERMINATE as usize, event_state == KeyEventState::PRESSED)
			}
			Scancode::F =>
			{
				self.state.actions.set(Action::CURSOR_CAPTURE_TOGGLE as usize, event_state == KeyEventState::PRESSED)
			}
			_ =>
			{
				let statestr = if event_state == KeyEventState::PRESSED
				{
					"pressed"
				}
				else
				{
					"released"
				};
				println!("Unmapped key {} {}", scancode.name(), statestr);
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

	pub fn update_mouse_button(&mut self, button: MouseButton, event_state: KeyEventState)
	{
		let statestr = if event_state == KeyEventState::PRESSED
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
			MouseButton::Left =>
			{
				println!("Left mouse {}!", statestr);
			}
			MouseButton::Right =>
			{
				println!("Right mouse {}!", statestr);
			}
			MouseButton::Middle =>
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
		&mut self, consumer: Rc<RefCell<T>>, mouse_invert: (bool, bool), mouse_sensitivity: f32,
	)
	{
		consumer.borrow_mut().register_mouse_settings(mouse_invert, mouse_sensitivity);
		self.mouse_consumer = Some(consumer);
	}

	pub fn update_mouse_movement(&mut self, mouse_delta: (i32, i32))
	{
		self.state.mouse_delta.0 += mouse_delta.0;
		self.state.mouse_delta.1 += mouse_delta.1;
	}

	pub fn mouse_movement_tick(&mut self, cursor_captured: bool)
	{
		if self.state.mouse_delta == (0, 0)
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

		self.state.mouse_delta = (0, 0);
	}
}
