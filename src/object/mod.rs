mod camera;
pub mod draw;

pub use self::camera::Camera;
pub use self::draw::DrawObject;

use cgmath::prelude::*;
use cgmath::{Matrix4, Point3};
use ash::vk;

pub trait Drawable {
    /// Draws the given object.
    fn draw(
        &self,
        cmd_buf: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        view_matrix: &Matrix4<f32>,
        projection_matrix: &Matrix4<f32>,
    );
}

pub trait Position {
    /// Returns the position of the given object.
    fn get_position(&self) -> Point3<f32>;

    /// Sets the position of the given object.
    fn set_position(&mut self, position: Point3<f32>);

    /// Gets the distance between the given and passed objects.
    fn get_distance<T: Position>(&self, other: &T) -> f32 {
        let vec = other.get_position() - self.get_position();
        vec.dot(vec).sqrt()
    }
}
