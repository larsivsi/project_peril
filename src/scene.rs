use cgmath::{Matrix4, Point3};
use object::{DrawObject, Drawable};
use renderer::RenderState;
use ash::vk;

pub struct Scene {
    objects: Vec<DrawObject>,
}

impl Scene {
    pub fn new(rs: &RenderState) -> Scene {
        let _quad = DrawObject::new_quad(rs, Point3::new(0.0, 0.0, 0.0), 1.0, 1.0);
        let cuboid = DrawObject::new_cuboid(rs, Point3::new(2.0, 0.0, -4.0), 1.0, 2.0, 1.0);

        let mut scene = Scene {
            objects: Vec::new(),
        };

        scene.objects.push(cuboid);

        scene
    }

    pub fn draw(
        &self,
        cmd_buf: vk::CommandBuffer,
        pipeline_layout: vk::PipelineLayout,
        view_matrix: &Matrix4<f32>,
        projection_matrix: &Matrix4<f32>,
    ) {
        for object in self.objects.iter() {
            object.draw(cmd_buf, pipeline_layout, view_matrix, projection_matrix);
        }
    }
}
