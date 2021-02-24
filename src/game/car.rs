use crate::core::{Action, Drawable, InputConsumer, Material, Mesh, Transform, Transformable};
use bit_vec::BitVec;
use cgmath::prelude::*;
use cgmath::Vector3;
use std::rc::Rc;

pub struct Car
{
	force: Vector3<f32>,
	velocity: Vector3<f32>,
	mass: f32,
	transform: Transform,
	mesh: Rc<Mesh>,
	material: Rc<Material>,
}

impl Car
{
	pub fn new(mass: f32, mesh: Rc<Mesh>, material: Rc<Material>) -> Car
	{
		let car = Car {
			force: Vector3::new(0.0, 0.0, 0.0),
			velocity: Vector3::new(0.0, 0.0, 0.0),
			mass: mass,
			transform: Transform::new(),
			mesh: mesh,
			material: material,
		};
		return car;
	}

	fn accelerate(&mut self, force: f32)
	{
		self.force += self.get_front_vector() * force;
	}

	fn decelerate(&mut self, force: f32)
	{
		self.accelerate(-force);
	}

	fn turn_left(&mut self, angle: f32)
	{
		self.yaw(angle);
	}

	fn turn_right(&mut self, angle: f32)
	{
		self.yaw(-angle);
	}

	pub fn update(&mut self)
	{
		// Drag
		let drag_coefficient = 20.0;
		self.force -= self.velocity * self.velocity.magnitude() * drag_coefficient;

		let acceleration = self.force / self.mass;

		// Reset force
		self.force = Vector3::new(0.0, 0.0, 0.0);

		// TODO ENGINE_TIMESTEP
		self.velocity += acceleration * 1.0 / 60.0;
		self.translate(self.velocity * 1.0 / 60.0);
	}
}

impl Transformable for Car
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

impl Drawable for Car
{
	fn get_mesh(&self) -> &Mesh
	{
		return &self.mesh;
	}
	fn get_material(&self) -> &Material
	{
		return &self.material;
	}
}

impl InputConsumer for Car
{
	fn get_handled_actions(&self) -> BitVec
	{
		let mut actions = BitVec::from_elem(Action::LENGTH_OF_ENUM as usize, false);
		actions.set(Action::FORWARD as usize, true);
		actions.set(Action::BACK as usize, true);
		actions.set(Action::LEFT as usize, true);
		actions.set(Action::RIGHT as usize, true);
		return actions;
	}
	fn consume(&mut self, actions: BitVec)
	{
		if actions.get(Action::FORWARD as usize).unwrap()
		{
			self.accelerate(100_000.0);
		}
		if actions.get(Action::BACK as usize).unwrap()
		{
			self.decelerate(100_000.0);
		}
		if actions.get(Action::LEFT as usize).unwrap()
		{
			self.turn_left(2.0);
		}
		if actions.get(Action::RIGHT as usize).unwrap()
		{
			self.turn_right(2.0);
		}
	}
}
