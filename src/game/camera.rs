use crate::core::{Action, InputConsumer, MouseConsumer, Transform, Transformable};
use bit_vec::BitVec;
use cgmath::{Point3, Vector3};

pub struct Camera
{
	mouse_invert: (bool, bool),
	mouse_sensitivity: f32,
	transform: Transform,
}

impl Camera
{
	pub fn new(position: Point3<f32>, front_vector: Vector3<f32>) -> Camera
	{
		let mut cam = Camera {
			mouse_invert: (false, false),
			mouse_sensitivity: 1.0,
			transform: Transform::new(),
		};
		cam.set_position(position);
		cam.set_initial_front_vector(front_vector);
		return cam;
	}
}

impl Transformable for Camera
{
	fn get_transform(&self) -> &Transform
	{
		return &self.transform;
	}
	fn get_mutable_transform(&mut self) -> &mut Transform
	{
		return &mut self.transform;
	}
}

impl InputConsumer for Camera
{
	fn get_handled_actions(&self) -> BitVec
	{
		let mut handled_actions = BitVec::from_elem(Action::LENGTH_OF_ENUM as usize, false);

		handled_actions.set(Action::SPRINT as usize, true);
		handled_actions.set(Action::FORWARD as usize, true);
		handled_actions.set(Action::LEFT as usize, true);
		handled_actions.set(Action::BACK as usize, true);
		handled_actions.set(Action::RIGHT as usize, true);
		handled_actions.set(Action::UP as usize, true);
		handled_actions.set(Action::DOWN as usize, true);
		handled_actions.set(Action::CAM_UP as usize, true);
		handled_actions.set(Action::CAM_LEFT as usize, true);
		handled_actions.set(Action::CAM_DOWN as usize, true);
		handled_actions.set(Action::CAM_RIGHT as usize, true);

		return handled_actions;
	}
	fn consume(&mut self, actions: BitVec)
	{
		let mut move_speed = 0.3;
		if actions.get(Action::SPRINT as usize).unwrap()
		{
			move_speed *= 10.0;
		}

		if actions.get(Action::FORWARD as usize).unwrap()
		{
			let translation = self.get_front_vector();
			self.translate(translation * move_speed);
		}
		if actions.get(Action::LEFT as usize).unwrap()
		{
			let translation = self.get_right_vector() * -1.0;
			self.translate(translation * move_speed);
		}
		if actions.get(Action::BACK as usize).unwrap()
		{
			let translation = self.get_front_vector() * -1.0;
			self.translate(translation * move_speed);
		}
		if actions.get(Action::RIGHT as usize).unwrap()
		{
			let translation = self.get_right_vector();
			self.translate(translation * move_speed);
		}
		if actions.get(Action::UP as usize).unwrap()
		{
			let translation = Vector3::unit_y();
			self.translate(translation * move_speed);
		}
		if actions.get(Action::DOWN as usize).unwrap()
		{
			let translation = Vector3::unit_y() * -1.0;
			self.translate(translation * move_speed);
		}
		if actions.get(Action::CAM_UP as usize).unwrap()
		{
			self.pitch(5.0);
		}
		if actions.get(Action::CAM_LEFT as usize).unwrap()
		{
			self.yaw(5.0);
		}
		if actions.get(Action::CAM_DOWN as usize).unwrap()
		{
			self.pitch(-5.0);
		}
		if actions.get(Action::CAM_RIGHT as usize).unwrap()
		{
			self.yaw(-5.0);
		}
	}
}

impl MouseConsumer for Camera
{
	fn register_mouse_settings(&mut self, mouse_invert: (bool, bool), mouse_sensitivity: f32)
	{
		self.mouse_invert = mouse_invert;
		self.mouse_sensitivity = mouse_sensitivity;
	}

	fn consume(&mut self, mouse_delta: (i32, i32))
	{
		let mut mouse_yaw = mouse_delta.0 as f32;
		let mut mouse_pitch = mouse_delta.1 as f32;
		let (x_invert, y_invert) = self.mouse_invert;
		// Yaw and pitch will be in the opposite direction of mouse delta
		mouse_yaw *= if x_invert
		{
			self.mouse_sensitivity
		}
		else
		{
			-self.mouse_sensitivity
		};
		mouse_pitch *= if y_invert
		{
			self.mouse_sensitivity
		}
		else
		{
			-self.mouse_sensitivity
		};

		self.yaw(mouse_yaw);
		self.pitch(mouse_pitch);
	}
}
