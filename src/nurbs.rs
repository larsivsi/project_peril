use cgmath::prelude::*;
use cgmath::Point3;

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum Order {
    LINEAR = 2,
    QUADRATIC = 3,
    CUBIC = 4,
    QUARTIC = 5,
}

pub struct NURBSpline {
    order: Order,
    controlpoints: Vec<Point3<f64>>,
    knots: Vec<f64>,
}

impl NURBSpline {
    /// Initializes a new NURBSpline based on input control points.
    ///
    /// This includes generating a knot vector etc.
    pub fn new(order: Order, controlpoints: Vec<Point3<f64>>) -> NURBSpline {
        debug_assert!(order as usize <= controlpoints.len());
        let knots_size = controlpoints.len() + order as usize;

        let mut spline = NURBSpline {
            order: order,
            controlpoints: controlpoints,
            knots: Vec::with_capacity(knots_size),
        };
        spline.generate_knots();

        spline
    }

    /// Returns the evaluation limit for the NURBSpline.
    ///
    /// The spline cannot be evaluated at any point equal to or greater than this limit.
    pub fn eval_limit(&self) -> f64 {
        // Value at the end of knots vector is the exclusive limit for
        // what values one can evaluate the NURBS with.
        self.knots[self.knots.len() - 1]
    }

    /// Evaluates the NURBSpline at the given value.
    ///
    /// This value has to be less than the evaluation limit for the spline.
    pub fn evaluate_at(&self, u: f64) -> Point3<f64> {
        debug_assert!(u < self.eval_limit());

        let mut result = Point3::new(0.0, 0.0, 0.0);
        let start_idx = u.floor() as usize;
        let order = self.order as usize;

        for idx in start_idx..(start_idx + order) {
            let contrib = self.coxdeboor(idx, order, u);
            let controlpoint = self.controlpoints[idx];
            result += (contrib * controlpoint).to_vec();
        }
        result
    }

    /// Cox-de Boor recursion formula.
    ///
    /// This returns the contribution of the given control point index, order and value to
    /// evaluate.
    /// See https://www.cs.montana.edu/paxton/classes/aui/dslectures/CoxdeBoor.pdf for details.
    fn coxdeboor(&self, idx: usize, order: usize, u: f64) -> f64 {
        debug_assert!(order > 0);

        if order == 1 {
            if self.knots[idx] <= u && u <= self.knots[idx + 1] {
                return 1.0;
            } else {
                return 0.0;
            }
        }

        let divident = self.knots[idx + order - 1] - self.knots[idx];
        let equation1 = if divident > 0.0 {
            (u - self.knots[idx]) / divident * self.coxdeboor(idx, order - 1, u)
        } else {
            0.0
        };

        let divident = self.knots[idx + order] - self.knots[idx + 1];
        let equation2 = if divident > 0.0 {
            (self.knots[idx + order] - u) / divident * self.coxdeboor(idx + 1, order - 1, u)
        } else {
            0.0
        };

        return equation1 + equation2;
    }

    /// Generates an open uniform knot vector.
    ///
    /// Refer to the pdf in the coxdeboor-documentation for details.
    fn generate_knots(&mut self) {
        let mut val = 0.0;
        let step = 1.0;
        let order = self.order as usize;

        // #order zeroes
        for _i in 0..order {
            self.knots.push(val);
        }
        val += step;
        // monotonically increasing knots
        for _i in 0..(self.controlpoints.len() - order) {
            self.knots.push(val);
            val += step;
        }
        // #order end values
        for _i in 0..order {
            self.knots.push(val);
        }
    }
}
