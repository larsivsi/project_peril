use draw_object::{Drawable, Position, DrawObject};

pub struct Scene {
    objects: Vec<DrawObject>,
}

impl Scene {
    pub fn new() -> Scene {
        let q1 = DrawObject::new((1.0, 0.0, 0.0));
        let q2 = DrawObject::new((2.0, 2.0, 0.0));

        let mut scene = Scene { objects: Vec::new() };

        scene.objects.push(q1);
        scene.objects.push(q2);

        scene
    }

    pub fn draw(&self) {
        for object in self.objects.iter() {
            println!("Would draw {:?}", object);
            object.draw();
        }
    }
}
