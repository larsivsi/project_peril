use cgmath::{Point3, Vector3};
use object::transform::{Transform, Transformable};

pub struct Camera
{
	transform: Transform,
}

impl Camera
{
	pub fn new(position: Point3<f32>) -> Camera
	{
		let mut cam = Camera {
			transform: Transform::new(),
		};
		cam.set_position(position);
		cam.set_initial_front_vector(-Vector3::unit_z());
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
