mod camera;
mod nurbs;
mod scene;

pub use self::camera::Camera;
pub use self::nurbs::{NURBSpline, Order};
pub use self::scene::Scene;

pub trait Game
{
}
