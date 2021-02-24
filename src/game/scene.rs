use crate::core::{ActionType, Config, Drawable, InputHandler, Material, Mesh, Transform, Transformable};
use crate::game::{Camera, Car, NURBSpline, Order};
use crate::renderer::{MainPass, RenderState};
use ash::{vk, Device};
use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Quaternion, Vector3};
use std::cell::RefCell;
use std::rc::Rc;

struct StaticObject
{
	transform: Transform,
	mesh: Rc<Mesh>,
	material: Rc<Material>,
}

impl StaticObject
{
	fn new(mesh: Rc<Mesh>, material: Rc<Material>) -> StaticObject
	{
		let obj = StaticObject {
			transform: Transform::new(),
			mesh: mesh,
			material: material,
		};
		return obj;
	}
}

impl Transformable for StaticObject
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

impl Drawable for StaticObject
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

struct SpinningCube
{
	transform: Transform,
	mesh: Rc<Mesh>,
	material: Rc<Material>,
}

impl SpinningCube
{
	fn new(mesh: Rc<Mesh>, material: Rc<Material>) -> SpinningCube
	{
		let obj = SpinningCube {
			transform: Transform::new(),
			mesh: mesh,
			material: material,
		};
		return obj;
	}

	fn update(&mut self)
	{
		self.globally_rotate(Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Deg(-0.5)));
		self.scale(1.001);
	}
}

impl Transformable for SpinningCube
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

impl Drawable for SpinningCube
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

pub struct Scene
{
	camera: Rc<RefCell<Camera>>,
	static_stuff: Vec<StaticObject>,
	spinning_cube: SpinningCube,
	car: Rc<RefCell<Car>>,
}

impl Scene
{
	pub fn new(rs: &RenderState, mp: &MainPass, cfg: &Config, input_handler: &mut InputHandler) -> Scene
	{
		let camera = Rc::new(RefCell::new(Camera::new(Point3::new(0.0, 10.0, 0.0), -Vector3::unit_z())));
		// input_handler.register_actions(camera.clone(), ActionType::TICK);
		input_handler.register_mouse_movement(
			camera.clone(),
			(cfg.mouse_invert_x, cfg.mouse_invert_y),
			cfg.mouse_sensitivity,
		);

		let metal_panel_surface = Material::new(
			rs,
			mp,
			"assets/thirdparty/textures/Metal_Panel_004/Metal_Panel_004_COLOR.jpg",
			"assets/thirdparty/textures/Metal_Panel_004/Metal_Panel_004_NORM.jpg",
		);
		let cube_surface = Material::new(
			rs,
			mp,
			"assets/original/textures/cubemap.png",
			"assets/original/textures/cubemap_normals.png",
		);

		let mut static_stuff = Vec::new();

		let floor_mesh = Mesh::new_quad(rs, 1_000.0, 1_000.0);
		let mut floor = StaticObject::new(floor_mesh.clone(), metal_panel_surface.clone());
		floor.globally_rotate(Quaternion::from_axis_angle(Vector3::new(-1.0, 0.0, 0.0), Deg(90.0)));
		static_stuff.push(floor);

		let cuboid_mesh = Mesh::new_cuboid(rs, 2.0, 2.0, 2.0);
		let mut spinning_cube = SpinningCube::new(cuboid_mesh, cube_surface.clone());
		spinning_cube.set_position(Point3::new(0.0, 5.0, -4.0));

		// Some standard car numbers (1.8m wide, 1.5m tall, 4.3m long, 1524kg)
		let car_mesh = Mesh::new_cuboid(rs, 1.8, 1.5, 4.3);
		let car = Rc::new(RefCell::new(Car::new(1_524.0, car_mesh, cube_surface.clone())));
		car.borrow_mut().set_position(Point3::new(0.0, 0.75, 0.0));
		input_handler.register_actions(car.clone(), ActionType::TICK);

		// For now, this is just done to not have the code unused.
		let points = vec![
			Point3::new(1.0, 0.0, 0.0),
			Point3::new(0.0, 1.0, 0.0),
			Point3::new(-1.0, 0.0, 0.0),
			Point3::new(0.0, -1.0, 0.0),
			Point3::new(0.0, 0.0, 1.0),
			Point3::new(0.0, 0.0, -1.0),
			Point3::new(0.0, 1.0, -1.0),
			Point3::new(1.0, 0.0, -1.0),
		];

		let mut u = 0.0;
		let step = 0.1;
		let spline = NURBSpline::new(Order::CUBIC, points);

		while u < spline.eval_limit()
		{
			let _point = spline.evaluate_at(u);
			u += step;
		}

		let scene = Scene {
			camera: camera,
			static_stuff: static_stuff,
			spinning_cube: spinning_cube,
			car: car,
		};

		return scene;
	}

	pub fn get_view_matrix(&mut self) -> Matrix4<f32>
	{
		return self.camera.borrow().generate_view_matrix();
	}

	pub fn update(&mut self)
	{
		self.spinning_cube.update();
		self.car.borrow_mut().update();
	}

	pub fn draw(
		&mut self, device: &Device, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout,
		view_matrix: &Matrix4<f32>, projection_matrix: &Matrix4<f32>,
	)
	{
		for obj in &self.static_stuff
		{
			let model_matrix = obj.generate_transformation_matrix();
			obj.draw(device, cmd_buf, pipeline_layout, &model_matrix, view_matrix, projection_matrix);
		}
		let mut model_matrix = self.spinning_cube.generate_transformation_matrix();
		self.spinning_cube.draw(device, cmd_buf, pipeline_layout, &model_matrix, view_matrix, projection_matrix);

		model_matrix = self.car.borrow().generate_transformation_matrix();
		self.car.borrow().draw(device, cmd_buf, pipeline_layout, &model_matrix, view_matrix, projection_matrix);
	}
}
