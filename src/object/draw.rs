use cgmath::Point3;
use object::transform::{Transform, Transformable};
use object::{Drawable, Material, Mesh};
use std::rc::Rc;

pub struct DrawObject
{
	transform: Transform,
	mesh: Rc<Mesh>,
	material: Rc<Material>,
}

impl Drawable for DrawObject
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

impl Transformable for DrawObject
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

impl DrawObject
{
	pub fn new(position: Point3<f32>, mesh: Rc<Mesh>, material: Rc<Material>) -> DrawObject
	{
		let mut object = DrawObject {
			transform: Transform::new(),
			mesh: mesh,
			material: material,
		};
		object.set_position(position);

		return object;
	}
}
