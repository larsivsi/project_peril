use ash::{vk, Device};
use cgmath::prelude::*;
use cgmath::{Deg, Matrix4, Point3, Quaternion, Vector3};
use core::{
	ActionType, ComponentType, Config, DrawComponent, Drawable, GameObject, InputConsumer, InputHandler, Material,
	Mesh, TransformComponent, Transformable,
};
use game::{Camera, NURBSpline, Order};
use renderer::{MainPass, RenderState};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub struct Scene
{
	root: GameObject,
	camera: Rc<RefCell<Camera>>,
}

impl Scene
{
	pub fn new(rs: &RenderState, mp: &MainPass, cfg: &Config, input_handler: &mut InputHandler) -> Scene
	{
		let mut scene = Scene {
			root: GameObject::new(),
			camera: Rc::new(RefCell::new(Camera::new(Point3::new(0.0, 0.0, 0.0)))),
		};
		input_handler.register_actions(
			scene.camera.borrow().get_handled_actions(),
			ActionType::TICK,
			scene.camera.clone(),
		);
		input_handler.register_mouse_movement(
			scene.camera.clone(),
			(cfg.mouse_invert_x, cfg.mouse_invert_y),
			cfg.mouse_sensitivity,
		);

		let quad_mesh = Mesh::new_quad(rs, 20.0, 20.0);
		let cuboid_mesh = Mesh::new_cuboid(rs, 2.0, 2.0, 2.0);
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

		let mut cuboid = GameObject::new();
		let mut transform = TransformComponent::new();
		transform.set_position(Point3::new(0.0, 0.0, -4.0));
		cuboid.add_component(transform);
		cuboid.add_component(DrawComponent::new(cuboid_mesh, cube_surface));
		scene.root.add_child(cuboid);

		let points = vec![
			Point3::new(1.0, 0.0, 0.0),
			Point3::new(-1.0, 0.0, 0.0),
			Point3::new(0.0, 1.0, 0.0),
			Point3::new(0.0, -1.0, 0.0),
			Point3::new(0.0, 0.0, -1.0),
			Point3::new(0.0, 0.0, 1.0),
		];
		let directions = vec![
			Vector3::new(0.0, -1.0, 0.0),
			Vector3::new(0.0, 1.0, 0.0),
			Vector3::new(1.0, 0.0, 0.0),
			Vector3::new(-1.0, 0.0, 0.0),
			Vector3::new(0.0, 0.0, 1.0),
			Vector3::new(0.0, 0.0, 1.0),
		];
		let mut logical_cube_node = GameObject::new();
		logical_cube_node.add_component(TransformComponent::new());
		for i in 0..6
		{
			let x: f32 = points[i].x;
			let y: f32 = points[i].y;
			let z: f32 = points[i].z;
			let mut wall = GameObject::new();
			let mut transform = TransformComponent::new();
			transform.globally_rotate(Quaternion::from_axis_angle(directions[i], Deg(90.0)));
			if i == 5
			{
				transform.globally_rotate(Quaternion::new(0.0, 0.0, 1.0, 0.0));
			}
			transform.set_position(Point3::new(20. * x, 20. * y, 20. * z));

			wall.add_component(transform);
			wall.add_component(DrawComponent::new(quad_mesh.clone(), metal_panel_surface.clone()));
			logical_cube_node.add_child(wall);
		}
		scene.root.add_child(logical_cube_node);

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

		return scene;
	}

	pub fn get_view_matrix(&mut self) -> Matrix4<f32>
	{
		match self.camera.borrow_mut().object.get_component(ComponentType::TRANSFORM)
		{
			Some(comp) =>
			{
				let immutable_comp = comp.borrow();
				let transform_comp = immutable_comp.as_any().downcast_ref::<TransformComponent>().unwrap();
				return transform_comp.generate_view_matrix();
			}
			None => panic!("impossible"),
		}
	}

	pub fn update(&mut self)
	{
		// For now, we know the rotating cube will be the first child of root
		match self.root.children[0].get_component(ComponentType::TRANSFORM)
		{
			Some(comp) =>
			{
				let mut mutable_comp = comp.borrow_mut();
				let transform_comp = mutable_comp.as_mutable_any().downcast_mut::<TransformComponent>().unwrap();
				transform_comp.globally_rotate(Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Deg(-0.5)));
				transform_comp.scale(1.001);
			}
			None => (),
		}
	}

	pub fn draw(
		&mut self, device: &Device, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout,
		view_matrix: &Matrix4<f32>, projection_matrix: &Matrix4<f32>,
	)
	{
		let mut to_visit: VecDeque<&mut GameObject> = VecDeque::new();
		to_visit.push_back(&mut self.root);

		while to_visit.len() > 0
		{
			let node = to_visit.pop_front().unwrap();

			if node.has_component(ComponentType::DRAW)
			{
				let model_matrix;
				// All drawable objects will also have a transform component
				match node.get_component(ComponentType::TRANSFORM)
				{
					Some(comp) =>
					{
						let immutable_comp = comp.borrow();
						let transform_comp = immutable_comp.as_any().downcast_ref::<TransformComponent>().unwrap();
						model_matrix = transform_comp.generate_transformation_matrix();
					}
					None => panic!("Draw without transform!"),
				}

				match node.get_component(ComponentType::DRAW)
				{
					Some(comp) =>
					{
						let immutable_comp = comp.borrow();
						let draw_comp = immutable_comp.as_any().downcast_ref::<DrawComponent>().unwrap();
						draw_comp.draw(device, cmd_buf, pipeline_layout, &model_matrix, view_matrix, projection_matrix);
					}
					None => (),
				}
			}

			for child in node.children.iter_mut()
			{
				to_visit.push_back(child);
			}
		}
	}
}
