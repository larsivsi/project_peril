use cgmath::Point3;
use object::{Drawable, Position, DrawObject};

pub struct Scene {
    objects: Vec<DrawObject>,
}

impl Scene {
    pub fn new() -> Scene {
        //let q1 = DrawObject::new(Point3::new(0.0, 0.0, 0.0));
        //let q2 = DrawObject::new(Point3::new(1.0, 1.0, 1.0));

        //println!("distance: {}", q1.get_distance(&q2));

        let mut scene = Scene { objects: Vec::new() };

        //scene.objects.push(q1);
        //scene.objects.push(q2);

        scene
    }

    pub fn draw(&self) {
        for object in self.objects.iter() {
            object.draw();
        }
    }
}
