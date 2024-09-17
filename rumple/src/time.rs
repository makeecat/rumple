pub trait Timeout {
    fn is_over(&self) -> bool;

    /// Update the number of attempted samples of the configuration space, incrementing by `_n`.
    /// This includes both valid and invalid sampled states.
    fn update_sample_count(&mut self, _n: usize) {}

    /// Update the number of nodes in a configuration-space graph, incrementing by `_n`.
    /// This exclusively includes valid sampled states.
    fn update_node_count(&mut self, _n: usize) {}
}

/// A helper structure for generating a composite timeout of multiple conditions.
///
/// # Examples
///
/// ```
/// use rumple::time::{Timeout, Forever, LimitSamples, Any};
/// let tc0 = Forever;
/// let tc1 = LimitSamples::new(1000);
/// let composed = Any((tc0, tc1));
/// assert!(!composed.is_over());
/// ```
///
/// You can use the bitwise-or operator on any provided timeout for easy composition.
///
/// ```
/// use rumple::time::{Timeout, LimitSamples, LimitNodes, Any};
/// let tc0 = LimitNodes::new(100);
/// let tc1 = LimitSamples::new(1000);
/// let composed = tc0 | tc1;
/// assert!(!composed.is_over());
/// ```
pub struct Any<T>(pub T);

pub struct Forever;

#[cfg(feature = "std")]
pub use alarm::Alarm;

pub struct LimitSamples {
    current: usize,
    limit: usize,
}

pub struct LimitNodes {
    current: usize,
    limit: usize,
}

/// Implement Timeout for an Any of some tuple.
macro_rules! any_tuple {
    () => {}; // This case is handled above by the trivial case
    ($($args:ident),*) => {
        impl<$($args: Timeout),*> Timeout for Any<($($args,)*)> {
            fn is_over(&self) -> bool {
                #[allow(non_snake_case)]
                let &($(ref $args,)*) = &self.0;
                $(
                    if ($args).is_over() {
                        return true;
                    }
                )*
                false
            }

            fn update_sample_count(&mut self, n: usize) {
                #[allow(non_snake_case)]
                let &mut ($(ref mut $args,)*) = &mut self.0;
                $(
                    $args.update_sample_count(n);
                )*
            }


            fn update_node_count(&mut self, n: usize) {
                #[allow(non_snake_case)]
                let &mut ($(ref mut $args,)*) = &mut self.0;
                $(
                    $args.update_node_count(n);
                )*
            }
        }
    }
}

any_tuple!();
any_tuple!(A);
any_tuple!(A, B);
any_tuple!(A, B, C);
any_tuple!(A, B, C, D);
any_tuple!(A, B, C, D, E);
any_tuple!(A, B, C, D, E, F);
any_tuple!(A, B, C, D, E, F, G);
any_tuple!(A, B, C, D, E, F, G, H);
any_tuple!(A, B, C, D, E, F, G, H, I);
any_tuple!(A, B, C, D, E, F, G, H, I, J);

macro_rules! bitor_impl {
    ($t: ident) => {
        impl<R: Timeout> core::ops::BitOr<R> for $t {
            type Output = Any<($t, R)>;
            fn bitor(self, rhs: R) -> Self::Output {
                Any((self, rhs))
            }
        }
    };
}

#[cfg(feature = "std")]
mod alarm {
    use super::Any;
    use crate::time::Timeout;
    use core::time::Duration;
    use std::time::Instant;

    pub struct Alarm(Instant);

    impl Alarm {
        #[must_use]
        #[expect(clippy::missing_const_for_fn)]
        pub fn ending_at(t: Instant) -> Self {
            Self(t)
        }

        #[must_use]
        pub fn from_now(d: Duration) -> Self {
            Self(Instant::now() + d)
        }

        #[must_use]
        pub fn secs_from_now(s: u64) -> Self {
            Self::from_now(Duration::from_secs(s))
        }
    }

    impl Timeout for Alarm {
        fn is_over(&self) -> bool {
            Instant::now() >= self.0
        }
    }

    bitor_impl!(Alarm);

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn fifty_millis() {
            let alarm = Alarm::from_now(Duration::from_millis(50));
            assert!(!alarm.is_over());
            std::thread::sleep(Duration::from_millis(50));
            assert!(alarm.is_over());
        }
    }
}

impl Timeout for Forever {
    fn is_over(&self) -> bool {
        false
    }
}

impl LimitNodes {
    #[must_use]
    pub const fn new(n: usize) -> Self {
        Self {
            current: 0,
            limit: n,
        }
    }
}

impl LimitSamples {
    #[must_use]
    pub const fn new(n: usize) -> Self {
        Self {
            current: 0,
            limit: n,
        }
    }
}

impl Timeout for LimitNodes {
    fn is_over(&self) -> bool {
        self.current >= self.limit
    }

    fn update_node_count(&mut self, n: usize) {
        self.current += n;
    }
}

impl Timeout for LimitSamples {
    fn is_over(&self) -> bool {
        self.current >= self.limit
    }

    fn update_sample_count(&mut self, n: usize) {
        self.current += n;
    }
}

bitor_impl!(Forever);
bitor_impl!(LimitSamples);
bitor_impl!(LimitNodes);
