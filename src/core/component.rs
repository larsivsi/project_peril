use crate::core::{Drawable, Material, Mesh, Transform, Transformable};
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
	fn get_component_type_static() -> ComponentType;
	fn get_component_type(&self) -> ComponentType;
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
	fn get_component_type_static() -> ComponentType
	{
		return ComponentType::DRAW;
	}

	fn get_component_type(&self) -> ComponentType
	{
		return ComponentType::DRAW;
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
	fn get_component_type_static() -> ComponentType
	{
		return ComponentType::TRANSFORM;
	}

	fn get_component_type(&self) -> ComponentType
	{
		return ComponentType::TRANSFORM;
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
