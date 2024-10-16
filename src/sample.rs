//! Sampling distributions for states.

use core::array;

use crate::space::{Angle, Pose2d, Vector};
use num_traits::{float::FloatCore, FloatConst};
use rand::{
    distributions::{uniform::SampleUniform, Bernoulli, Distribution},
    Rng,
};

/// A sampler for a configuration.
///
/// `C` is the type of the configuration, and `RNG` is a source of randomness.
pub trait Sample<C, RNG> {
    /// Sample a configuration, using `rng` as a source of randomness.
    fn sample(&self, rng: &mut RNG) -> C;
}

impl<A, B, S, RNG> Sample<(A, B), RNG> for S
where
    S: Sample<A, RNG> + Sample<B, RNG>,
{
    fn sample(&self, _: &mut RNG) -> (A, B) {
        todo!()
    }
}

impl<RNG: Rng> Sample<bool, RNG> for Bernoulli {
    fn sample(&self, rng: &mut RNG) -> bool {
        <Self as Distribution<bool>>::sample(self, rng)
    }
}

/// A rectangular prism of values of `T`.
/// When used as [`Sample`], this rectangle inclusively samples all values.
pub struct Rectangle<T> {
    /// The lowest corner along all axes.
    pub min: T,
    /// The highest corner along all axes.
    pub max: T,
}

/// A sampler for all possible values of a datum.
pub struct Everywhere;

impl<const N: usize, T, RNG: Rng> Sample<Vector<N, T>, RNG> for Rectangle<Vector<N, T>>
where
    T: FloatCore + SampleUniform,
{
    fn sample(&self, rng: &mut RNG) -> Vector<N, T> {
        Vector::new(array::from_fn(|i| rng.gen_range(self.min[i]..=self.max[i])))
    }
}

impl<T, RNG: Rng> Sample<Angle<T>, RNG> for Everywhere
where
    T: FloatCore + FloatConst + SampleUniform,
{
    fn sample(&self, rng: &mut RNG) -> Angle<T> {
        unsafe { Angle::new_unchecked(rng.gen_range(T::zero()..T::TAU())) }
    }
}

impl<T, RNG: Rng> Sample<Pose2d<T>, RNG> for Rectangle<Vector<2, T>>
where
    T: FloatConst + FloatCore + SampleUniform,
{
    fn sample(&self, rng: &mut RNG) -> Pose2d<T> {
        Pose2d {
            position: self.sample(rng),
            angle: Everywhere.sample(rng),
        }
    }
}
