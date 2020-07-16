use crate::core::{Drawable, Material, Mesh, Transform, Transformable};
use std::any::Any;
use std::rc::Rc;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ComponentType
{
	DRAW,
	TRANSFORM,
	LENGTH_OF_ENUM,
}

pub trait Component
{
	fn get_component_type(&self) -> ComponentType;
	fn as_any(&self) -> &dyn Any;
	fn as_mutable_any(&mut self) -> &mut dyn Any;
}

pub struct DrawComponent
{
	mesh: Rc<Mesh>,
	material: Rc<Material>,
}

impl DrawComponent
{
	pub fn new(mesh: Rc<Mesh>, material: Rc<Material>) -> DrawComponent
	{
		return DrawComponent {
			mesh: mesh,
			material: material,
		};
	}
}

impl Component for DrawComponent
{
	fn get_component_type(&self) -> ComponentType
	{
		return ComponentType::DRAW;
	}
	fn as_any(&self) -> &dyn Any
	{
		return self;
	}
	fn as_mutable_any(&mut self) -> &mut dyn Any
	{
		return self;
	}
}

impl Drawable for DrawComponent
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

pub struct TransformComponent
{
	transform: Transform,
}

impl TransformComponent
{
	pub fn new() -> TransformComponent
	{
		return TransformComponent {
			transform: Transform::new(),
		};
	}
}

impl Component for TransformComponent
{
	fn get_component_type(&self) -> ComponentType
	{
		return ComponentType::TRANSFORM;
	}
	fn as_any(&self) -> &dyn Any
	{
		return self;
	}
	fn as_mutable_any(&mut self) -> &mut dyn Any
	{
		return self;
	}
}

impl Transformable for TransformComponent
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
