use cgmath::{Matrix4, Point3, Quaternion, Vector3};
use object::{DrawObject, Drawable, Rotation};
use renderer::RenderState;
use ash::vk;
use std::f32;
use std::ops::MulAssign;

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

    pub fn update(&mut self) {
        for mut object in self.objects.iter_mut() {
            //TODO: Move this.
            let mut vec = Vector3::new(0.0, 0.0, 1.0);
            let rotation = f32::consts::PI / 2.0;
            let sine = rotation.sin();
            let cose = rotation.cos();
            vec.mul_assign(sine);
            let quaternion = Quaternion::new(vec.x, vec.y, vec.z, cose);

            object.rotate(quaternion);
        }
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
