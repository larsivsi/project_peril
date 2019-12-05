use object::transform::{Transform, Transformable};
use object::{Drawable, Material, Mesh};
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

pub trait Component: Any
{
	fn get_component_type(&self) -> ComponentType;
	fn get(&self) -> &dyn Any;
	fn get_mutable(&mut self) -> &mut dyn Any;
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
	fn get(&self) -> &dyn Any
	{
		return self;
	}
	fn get_mutable(&mut self) -> &mut dyn Any
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
	fn get(&self) -> &dyn Any
	{
		return self;
	}
	fn get_mutable(&mut self) -> &mut dyn Any
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
