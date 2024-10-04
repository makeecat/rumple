use crate::space::Angle;

use super::{Aabb, Ball};
use alloc::vec::Vec;
use num_traits::{float::FloatCore, FloatConst};
pub struct World2d<T = f64> {
    aabbs: Vec<Aabb<2, T>>,
    balls: Vec<Ball<2, T>>,
}

impl<T> Default for World2d<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> World2d<T> {
    #[must_use]
    /// Create a new empty world.
    pub const fn new() -> Self {
        Self {
            aabbs: Vec::new(),
            balls: Vec::new(),
        }
    }
}

impl<T> World2d<T>
where
    T: FloatCore,
{
    pub fn collides_ball(&self, x: T, y: T, r: T) -> bool {
        debug_assert!(T::zero() <= r, "radius of ball must be positive");
        // todo use SIMD
        self.aabbs.iter().any(
            |&Aabb {
                 los: [lx, ly],
                 his: [hx, hy],
             }| {
                let nx = x.clamp(lx, hx);
                let ny = y.clamp(ly, hy);
                ((nx - x).powi(2) + (ny - y).powi(2)) <= r.powi(2)
            },
        ) || self.balls.iter().any(
            |&Ball {
                 pos: [xb, yb],
                 r: rb,
             }| {
                let xdiff = xb - x;
                let ydiff = yb - y;
                let rpsq = rb + r;
                xdiff * xdiff + ydiff * ydiff <= rpsq * rpsq
            },
        )
    }

    pub fn collides_point(&self, x: T, y: T) -> bool {
        self.aabbs.iter().any(
            |&Aabb {
                 los: [lx, ly],
                 his: [hx, hy],
             }| x >= lx && x <= hx && y >= ly && y <= hy,
        ) || self.balls.iter().any(|&Ball { pos: [xb, yb], r }| {
            let xdiff = xb - x;
            let ydiff = yb - y;
            xdiff * xdiff + ydiff * ydiff <= r * r
        })
    }

    pub fn add_ball(&mut self, x: T, y: T, r: T) {
        debug_assert!(r >= T::zero(), "ball must have positive radius");
        self.balls.push(Ball { pos: [x, y], r });
    }

    pub fn add_aabb(&mut self, xl: T, yl: T, xh: T, yh: T) {
        debug_assert!(T::zero() <= xh - xl, "aabb must have positive width");
        debug_assert!(T::zero() <= yh - yl, "aabb must have positive height");
        self.aabbs.push(Aabb {
            los: [xl, yl],
            his: [xl, yl],
        });
    }
}

#[cfg(feature = "std")]
impl<T> World2d<T>
where
    T: num_traits::Float + FloatConst + Copy + std::fmt::Debug,
{
    /// Determine whether a rectangle collides with any object in this world.
    /// Returns `true` if the rectangle is in collision and `false` otherwise.
    ///
    /// The rectangle is centered at position `(x, x)` and when oriented with `theta = 0` has width
    /// `2 * half_w` and height `2 * half_h`.
    ///
    /// # Panics
    ///
    /// This function may panic (but may also return an erroneous result) if `w < 0` or if `h < 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rumple::{env::World2d, space::Angle};
    /// let mut world = World2d::new();
    ///
    /// // create ball of radius 0.5 at position (1.0, 1.0)
    /// world.add_ball(1.0, 1.0, 0.5);
    ///
    /// // rectangle centered at (0.0, 1.0) with width 1.5 and height 0.25 collides with the ball
    /// assert!(world.collides_rect(0.0, 1.0, Angle::new(0.0), 1.5, 0.25));
    ///
    /// // but if we rotate the rectangle, it won't collide
    /// assert!(!world.collides_rect(0.0, 1.0, Angle::new(std::f64::consts::PI / 2.0), 0.75, 0.25));
    /// ```
    pub fn collides_rect(&self, x: T, y: T, theta: Angle<T>, half_w: T, half_h: T) -> bool {
        debug_assert!(
            T::zero() <= half_w,
            "width of rect for collision checking must be positive"
        );
        debug_assert!(
            T::zero() <= half_h,
            "height of rect for collision checking must be positive",
        );
        let cos = theta.get().cos();
        let sin = theta.get().sin();
        self.balls.iter().any(|&Ball { pos: [xc, yc], r }| {
            let delta_x = xc - x;
            let delta_y = yc - y;

            // transform to coordinate frame of rect
            // multiply by inverse rotation matrix
            let x_trans = delta_x * cos + delta_y * sin;
            let y_trans = -delta_x * sin + delta_y * cos;

            // (x_trans, y_trans) is the position of the center of the ball
            let x_clamp = x_trans.clamp(-half_w, half_w);
            let y_clamp = y_trans.clamp(-half_h, half_h);

            // compare to closest point in rectangle body
            let x_diff = x_clamp - x_trans;
            let y_diff = y_clamp - y_trans;

            // dbg!(xc, yc, delta_x, delta_y, x_trans, y_trans, x_clamp, y_clamp, x_diff, y_diff);

            x_diff * x_diff + y_diff * y_diff <= r * r
        }) || self
            .aabbs
            .iter()
            .any(|_| todo!("implement collision checking for AABB/rect"))
    }
}
