use cgmath::prelude::*;
use cgmath::Point3;

#[derive(Copy, Clone)]
pub enum Order {
    LINEAR = 2,
    QUADRATIC = 3,
    CUBIC = 4,
    QUARTIC = 5,
}

pub struct NURBSpline {
    order: Order,
    control_points: Vec<Point3<f64>>,
    knots: Vec<f64>,
}

impl NURBSpline {
    pub fn new(order: Order, control_points: Vec<Point3<f64>>, knot_step: f64) -> NURBSpline {
        debug_assert!(order as usize <= control_points.len());
        let knots_size = control_points.len() + order as usize;

        let mut spline = NURBSpline {
            order: order,
            control_points: control_points,
            knots: Vec::with_capacity(knots_size),
        };
        spline.generate_knots(knot_step);

        spline
    }

    pub fn evaluate_at(&self, time: f64) -> Point3<f64> {
        let mut res = Point3::new(0.0, 0.0, 0.0);
        //TODO: only have to evaluate #order points here
        for (idx, cp) in self.control_points.iter().enumerate() {
            let val = self.coxdeboor(idx, self.order as u32, time);
            println!("idx {} contributes with {}", idx, val);
            res += (val * cp).to_vec();
        }
        res
    }

    //Cox-de Boor recursion formula
    fn coxdeboor(&self, cp_idx: usize, order: u32, t: f64) -> f64 {
        debug_assert!(order > 0);

        if order == 1 {
            if self.knots[cp_idx] <= t && t <= self.knots[cp_idx + 1] {
                return 1.0;
            } else {
                return 0.0;
            }
        }

        let divident = self.knots[cp_idx + order as usize - 1] - self.knots[cp_idx];
        let equation1 = if divident > 0.0 {
            (t - self.knots[cp_idx]) / divident * self.coxdeboor(cp_idx, order - 1, t)
        } else {
            0.0
        };

        let divident = self.knots[cp_idx + order as usize] - self.knots[cp_idx + 1];
        let equation2 = if divident > 0.0 {
            (self.knots[cp_idx + order as usize] - t) / divident *
                self.coxdeboor(cp_idx + 1, order - 1, t)
        } else {
            0.0
        };

        return equation1 + equation2;
    }

    // generates an open uniform knot vector
    fn generate_knots(&mut self, step: f64) {
        let mut val = 0.0;
        println!("knots!");
        // #order zeroes to start
        for _i in 0..self.order as usize {
            self.knots.push(val);
            println!("{}", val);
        }
        val += step;
        // monotonically increasing knots
        for _i in 0..(self.control_points.len() - self.order as usize) {
            self.knots.push(val);
            println!("{}", val);
            val += step;
        }
        // #order
        for _i in 0..self.order as usize {
            self.knots.push(val);
            println!("{}", val);
        }
    }
}
