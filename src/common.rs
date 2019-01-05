use std::hash::{BuildHasher};
use std::collections::hash_map::DefaultHasher;
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

macro_rules! verbose_log {
    ($tick_index:expr, $message:tt) => {
        if cfg!(feature = "enable_verbose_log") {
            log!($tick_index, $message);
        }
    };
    ($tick_index:expr, $format:tt, $($value:expr),*) => {
        if cfg!(feature = "enable_verbose_log") {
            log!($tick_index, $format, $($value),*);
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
    type Hasher = DefaultHasher;

    #[inline]
    fn build_hasher(&self) -> DefaultHasher {
        DefaultHasher::new()
    }
}

pub struct IdGenerator {
    next: i32,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator {next: 1}
    }

    pub fn next(&mut self) -> i32 {
        let result = self.next;
        self.next += 1;
        result
    }
}

pub fn as_score(value: f64) -> i32 {
    (value * 1000.0).round() as i32
}
