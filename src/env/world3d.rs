use alloc::vec::Vec;
use num_traits::float::FloatCore;

#[cfg(feature = "simd")]
use std::simd::{LaneCount, Simd, SupportedLaneCount};

use super::{Aabb, Ball};

pub struct World3d<T> {
    balls: Vec<Ball<3, T>>,
    aabbs: Vec<Aabb<3, T>>,
}

impl<T> World3d<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            balls: Vec::new(),
            aabbs: Vec::new(),
        }
    }

    pub fn add_ball(&mut self, x: T, y: T, z: T, r: T) {
        self.balls.push(Ball { pos: [x, y, z], r });
    }

    pub fn add_aabb(&mut self, xl: T, yl: T, zl: T, xh: T, yh: T, zh: T) {
        self.aabbs.push(Aabb {
            los: [xl, yl, zl],
            his: [xh, yh, zh],
        });
    }

    pub fn collides_ball(&self, x: T, y: T, z: T, r: T) -> bool
    where
        T: FloatCore,
    {
        let rsq = r * r;
        self.balls.iter().any(
            |&Ball {
                 pos: [xb, yb, zb],
                 r: rb,
             }| {
                let xdiff = xb - x;
                let ydiff = yb - y;
                let zdiff = zb - z;
                let rplus = rb + r;
                xdiff * xdiff + ydiff * ydiff + zdiff * zdiff <= rplus * rplus
            },
        ) || self.aabbs.iter().any(
            |&Aabb {
                 los: [xl, yl, zl],
                 his: [xh, yh, zh],
             }| {
                let xdiff = if x < xl {
                    xl - x
                } else if x > xh {
                    x - xh
                } else {
                    T::zero()
                };

                let ydiff = if y < yl {
                    yl - y
                } else if y > yh {
                    y - yh
                } else {
                    T::zero()
                };

                let zdiff = if z < zl {
                    zl - z
                } else if z > zh {
                    z - zh
                } else {
                    T::zero()
                };

                xdiff * xdiff + ydiff * ydiff + zdiff * zdiff <= rsq
            },
        )
    }
}

macro_rules! simd_impl {
    ($name: ident, $t: ty) => {
        #[cfg(feature = "simd")]
        impl World3d<$t> {
            pub fn $name<const L: usize>(
                &self,
                xs: Simd<$t, L>,
                ys: Simd<$t, L>,
                zs: Simd<$t, L>,
                rs: Simd<$t, L>,
            ) -> bool
            where
                LaneCount<L>: SupportedLaneCount,
            {
                use std::simd::cmp::SimdPartialOrd;

                let rsqs = rs * rs;
                self.balls.iter().any(
                    |&Ball {
                         pos: [xb, yb, zb],
                         r: rb,
                     }| {
                        let xdiff = Simd::splat(xb) - xs;
                        let ydiff = Simd::splat(yb) - ys;
                        let zdiff = Simd::splat(zb) - zs;
                        let rplus = Simd::splat(rb) + rs;
                        (xdiff * xdiff + ydiff * ydiff + zdiff * zdiff)
                            .simd_le(rplus * rplus)
                            .any()
                    },
                ) || self.aabbs.iter().any(
                    |&Aabb {
                         los: [xl, yl, zl],
                         his: [xh, yh, zh],
                     }| {
                        let xl = Simd::splat(xl);
                        let xh = Simd::splat(xh);
                        let xdiff = xs
                            .simd_lt(xl)
                            .select(xl - xs, xs.simd_gt(xh).select(xs - xh, Simd::splat(0.0)));

                        let yl = Simd::splat(yl);
                        let yh = Simd::splat(yh);
                        let ydiff = ys
                            .simd_lt(yl)
                            .select(yl - ys, ys.simd_gt(yh).select(ys - yh, Simd::splat(0.0)));

                        let zl = Simd::splat(zl);
                        let zh = Simd::splat(zh);
                        let zdiff = zs
                            .simd_lt(zl)
                            .select(zl - zs, zs.simd_gt(zh).select(zs - zh, Simd::splat(0.0)));

                        (xdiff * xdiff + ydiff * ydiff + zdiff * zdiff)
                            .simd_le(rsqs)
                            .any()
                    },
                )
            }
        }
    };
}

simd_impl!(collides_balls_f32, f32);
simd_impl!(collides_balls_f64, f64);

impl<T> Default for World3d<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "simd")]
    #[test]
    fn try_simd() {
        use std::simd::Simd;

        use crate::env::World3d;

        let xs = Simd::from_array([0.0, 1.0]);
        let ys = Simd::from_array([0.0, 1.0]);
        let zs = Simd::from_array([0.0, 1.0]);
        let rs = Simd::splat(0.5);

        let mut world = World3d::new();

        assert!(!world.collides_balls_f64(xs, ys, zs, rs));
        world.add_ball(1.25, 1.0, 1.0, 0.25);
        assert!(world.collides_balls_f64(xs, ys, zs, rs));

        world = World3d::new();

        world.add_aabb(-0.1, -0.1, -0.1, 0.1, 0.1, 0.1);
        assert!(world.collides_balls_f64(xs, ys, zs, rs));
    }
}
