use cgmath::Point3;
use object::{DrawObject, Drawable};
use renderer::RenderState;
use ash::vk;

pub struct Scene {
    objects: Vec<DrawObject>,
}

impl Scene {
    pub fn new(rs: &RenderState) -> Scene {
        let quad = DrawObject::new_quad(rs, Point3::new(0.0, 0.0, 0.0), 1.0, 1.0);

        let cuboid = DrawObject::new_cuboid(rs, Point3::new(0.0, 0.0, 4.0), 1.0, 2.0, 1.0);

        let mut scene = Scene {
            objects: Vec::new(),
        };

        scene.objects.push(quad);
        scene.objects.push(cuboid);

        scene
    }

    #[allow(dead_code)] //will be utilized in the future
    pub fn draw(&self, cmd_buf: vk::CommandBuffer) {
        for object in self.objects.iter() {
            object.draw(cmd_buf);
        }
    }
}
