mod camera;
mod draw;
mod material;
pub mod mesh;
pub mod transform;

pub use self::camera::Camera;
pub use self::draw::DrawObject;
pub use self::material::Material;
pub use self::mesh::Mesh;

use ash::{vk, Device};
use cgmath::Matrix4;

pub trait Drawable
{
	fn get_mesh(&self) -> &Mesh;
	fn get_material(&self) -> &Material;
	fn draw(
		&self, device: &Device, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout,
		view_matrix: &Matrix4<f32>, projection_matrix: &Matrix4<f32>,
	);
}
