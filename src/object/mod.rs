mod camera;
pub mod draw;
pub mod transform;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

use ash::vk;
use cgmath::Matrix4;

pub trait Drawable
{
	/// Draws the given object.
	fn draw(
		&self, cmd_buf: vk::CommandBuffer, pipeline_layout: vk::PipelineLayout, view_matrix: &Matrix4<f32>,
		projection_matrix: &Matrix4<f32>,
	);
}
