use std::ops::{Mul};

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
