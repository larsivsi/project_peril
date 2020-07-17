use crate::core::{Action, GameObject, InputConsumer, MouseConsumer, TransformComponent, Transformable};
use bit_vec::BitVec;
use cgmath::{Point3, Vector3};

pub struct Camera
{
	pub object: GameObject,
	mouse_invert: (bool, bool),
	mouse_sensitivity: f64,
}

impl Camera
{
	pub fn new(position: Point3<f32>) -> Camera
	{
		let mut cam = Camera {
			object: GameObject::new(),
			mouse_invert: (false, false),
			mouse_sensitivity: 1.0,
		};
		let mut transform = TransformComponent::new();
		transform.set_position(position);
		transform.set_initial_front_vector(-Vector3::unit_z());
		cam.object.add_component(transform);
		return cam;
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

		if let Some(transform_comp) = self.object.get_component::<TransformComponent>()
		{
			if actions.get(Action::FORWARD as usize).unwrap()
			{
				let translation = transform_comp.get_front_vector();
				transform_comp.translate(translation * move_speed);
			}
			if actions.get(Action::LEFT as usize).unwrap()
			{
				let translation = transform_comp.get_right_vector() * -1.0;
				transform_comp.translate(translation * move_speed);
			}
			if actions.get(Action::BACK as usize).unwrap()
			{
				let translation = transform_comp.get_front_vector() * -1.0;
				transform_comp.translate(translation * move_speed);
			}
			if actions.get(Action::RIGHT as usize).unwrap()
			{
				let translation = transform_comp.get_right_vector();
				transform_comp.translate(translation * move_speed);
			}
			if actions.get(Action::UP as usize).unwrap()
			{
				let translation = Vector3::unit_y();
				transform_comp.translate(translation * move_speed);
			}
			if actions.get(Action::DOWN as usize).unwrap()
			{
				let translation = Vector3::unit_y() * -1.0;
				transform_comp.translate(translation * move_speed);
			}
			if actions.get(Action::CAM_UP as usize).unwrap()
			{
				transform_comp.pitch(5.0);
			}
			if actions.get(Action::CAM_LEFT as usize).unwrap()
			{
				transform_comp.yaw(5.0);
			}
			if actions.get(Action::CAM_DOWN as usize).unwrap()
			{
				transform_comp.pitch(-5.0);
			}
			if actions.get(Action::CAM_RIGHT as usize).unwrap()
			{
				transform_comp.yaw(-5.0);
			}
		}
	}
}

impl MouseConsumer for Camera
{
	fn register_mouse_settings(&mut self, mouse_invert: (bool, bool), mouse_sensitivity: f64)
	{
		self.mouse_invert = mouse_invert;
		self.mouse_sensitivity = mouse_sensitivity;
	}

	fn consume(&mut self, mouse_delta: (f64, f64))
	{
		let (mut mouse_yaw, mut mouse_pitch) = mouse_delta;
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

		if let Some(transform_comp) = self.object.get_component::<TransformComponent>()
		{
			transform_comp.yaw(mouse_yaw as f32);
			transform_comp.pitch(mouse_pitch as f32);
		}
	}
}
