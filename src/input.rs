use bit_vec::BitVec;
use winit::ElementState;
use winit::KeyboardInput;

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

pub enum KeyIndex
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
	TERMINATE,
	CURSOR_CAPTURE,
	LENGTH_OF_ENUM,
}

pub struct InputState
{
	keys: BitVec,
}

impl InputState
{
	pub fn new() -> InputState
	{
		InputState {
			keys: BitVec::from_elem(KeyIndex::LENGTH_OF_ENUM as usize, false),
		}
	}
	pub fn update_key(&mut self, event: KeyboardInput)
	{
		match event.scancode
		{
			W_SCAN_CODE => self.keys.set(KeyIndex::FORWARD as usize, event.state == ElementState::Pressed),
			A_SCAN_CODE => self.keys.set(KeyIndex::LEFT as usize, event.state == ElementState::Pressed),
			S_SCAN_CODE => self.keys.set(KeyIndex::BACK as usize, event.state == ElementState::Pressed),
			D_SCAN_CODE => self.keys.set(KeyIndex::RIGHT as usize, event.state == ElementState::Pressed),
			SPACE_SCAN_CODE => self.keys.set(KeyIndex::UP as usize, event.state == ElementState::Pressed),
			LCTRL_SCAN_CODE => self.keys.set(KeyIndex::DOWN as usize, event.state == ElementState::Pressed),
			LSHIFT_SCAN_CODE => self.keys.set(KeyIndex::SPRINT as usize, event.state == ElementState::Pressed),
			UP_SCAN_CODE =>
			{
				self.keys.set(KeyIndex::CAM_UP as usize, event.state == ElementState::Pressed)
				// camera.pitch(5.0);
			}
			LEFT_SCAN_CODE =>
			{
				self.keys.set(KeyIndex::CAM_LEFT as usize, event.state == ElementState::Pressed)
				// camera.yaw(5.0);
			}
			DOWN_SCAN_CODE =>
			{
				self.keys.set(KeyIndex::CAM_DOWN as usize, event.state == ElementState::Pressed)
				// camera.pitch(-5.0);
			}
			RIGHT_SCAN_CODE =>
			{
				self.keys.set(KeyIndex::CAM_RIGHT as usize, event.state == ElementState::Pressed)
				// camera.yaw(-5.0);
			}
			ESC_SCAN_CODE => self.keys.set(KeyIndex::TERMINATE as usize, event.state == ElementState::Pressed),
			F_SCAN_CODE =>
			{
				if event.state == ElementState::Pressed
				{
					self.keys.set(
						KeyIndex::CURSOR_CAPTURE as usize,
						!self.keys.get(KeyIndex::CURSOR_CAPTURE as usize).unwrap(),
					);
				}
			}
			_ =>
			{
				println!("Unmapped key {} pressed", event.scancode);
			}
		}
	}

	pub fn get(&self, bv_idx: KeyIndex) -> bool
	{
		self.keys.get(bv_idx as usize).unwrap()
	}
}
