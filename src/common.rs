use std::hash::{BuildHasher, Hash, Hasher, SipHasher};
use std::ops::Mul;

macro_rules! log {
    ($tick_index:expr, $message:tt) => {
        if cfg!(feature = "enable_log") {
            let f = || {
                use std::io::{stdout, Write};
                write!(&mut stdout(), "[{}] {}\n", $tick_index, $message).unwrap();
            };
            f();
        }
    };
    ($tick_index:expr, $format:tt, $($value:expr),*) => {
        if cfg!(feature = "enable_log") {
            let f = || {
                use std::io::{stdout, Write};
                write!(&mut stdout(), "[{}] {}\n", $tick_index, format!($format, $($value),*)).unwrap();
            };
            f();
        }
    };
}

pub trait Square: Mul + Copy {
    fn square(self) -> Self::Output {
        self * self
    }
}

impl Square for f64 {}

pub trait IsBetween: PartialOrd + Copy {
    fn is_between(self, left: Self, right: Self) -> bool {
        left < self && self < right
    }
}

impl IsBetween for f64 {}

pub trait Clamp: PartialOrd + Sized {
    fn clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

impl Clamp for f64 {}

#[derive(Debug, Default, Clone)]
pub struct ConstState;

impl BuildHasher for ConstState {
    type Hasher = SipHasher;

    #[inline]
    fn build_hasher(&self) -> SipHasher {
        SipHasher::new_with_keys(3841724797260025322, 16478027545562035801)
    }
}
