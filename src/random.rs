use std::char;
use std::marker;
use std::mem;
use std::num::Wrapping as w;
use std::time;

#[allow(bad_style)]
type w64 = w<u64>;
#[allow(bad_style)]
type w32 = w<u32>;

pub trait Sample<Support> {
    fn sample<R: Rng>(&mut self, rng: &mut R) -> Support;
}

pub trait IndependentSample<Support>: Sample<Support> {
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> Support;
}

pub trait Rand : Sized {
    fn rand<R: Rng>(rng: &mut R) -> Self;
}

pub trait Rng {
    fn next_u32(&mut self) -> u32;

    fn next_u64(&mut self) -> u64 {
        ((self.next_u32() as u64) << 32) | (self.next_u32() as u64)
    }

    fn next_f32(&mut self) -> f32 {
        const UPPER_MASK: u32 = 0x3F800000;
        const LOWER_MASK: u32 = 0x7FFFFF;
        let tmp = UPPER_MASK | (self.next_u32() & LOWER_MASK);
        let result: f32 = unsafe { mem::transmute(tmp) };
        result - 1.0
    }

    fn next_f64(&mut self) -> f64 {
        const UPPER_MASK: u64 = 0x3FF0000000000000;
        const LOWER_MASK: u64 = 0xFFFFFFFFFFFFF;
        let tmp = UPPER_MASK | (self.next_u64() & LOWER_MASK);
        let result: f64 = unsafe { mem::transmute(tmp) };
        result - 1.0
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let mut count = 0;
        let mut num = 0;
        for byte in dest.iter_mut() {
            if count == 0 {
                num = self.next_u64();
                count = 8;
            }

            *byte = (num & 0xff) as u8;
            num >>= 8;
            count -= 1;
        }
    }

    #[inline(always)]
    fn gen<T: Rand>(&mut self) -> T where Self: Sized {
        Rand::rand(self)
    }

    fn gen_iter<'a, T: Rand>(&'a mut self) -> Generator<'a, T, Self> where Self: Sized {
        Generator { rng: self, _marker: marker::PhantomData }
    }

    fn gen_range<T: PartialOrd + SampleRange>(&mut self, low: T, high: T) -> T where Self: Sized {
        assert!(low < high, "Rng.gen_range called with low >= high");
        Range::new(low, high).ind_sample(self)
    }

    fn gen_weighted_bool(&mut self, n: u32) -> bool where Self: Sized {
        n <= 1 || self.gen_range(0, n) == 0
    }

    fn gen_ascii_chars<'a>(&'a mut self) -> AsciiGenerator<'a, Self> where Self: Sized {
        AsciiGenerator { rng: self }
    }

    fn choose<'a, T>(&mut self, values: &'a [T]) -> Option<&'a T> where Self: Sized {
        if values.is_empty() {
            None
        } else {
            Some(&values[self.gen_range(0, values.len())])
        }
    }

    fn choose_mut<'a, T>(&mut self, values: &'a mut [T]) -> Option<&'a mut T> where Self: Sized {
        if values.is_empty() {
            None
        } else {
            let len = values.len();
            Some(&mut values[self.gen_range(0, len)])
        }
    }

    fn shuffle<T>(&mut self, values: &mut [T]) where Self: Sized {
        let mut i = values.len();
        while i >= 2 {
            i -= 1;
            values.swap(i, self.gen_range(0, i + 1));
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Range<X> {
    low: X,
    range: X,
    accept_zone: X
}

impl<X: SampleRange + PartialOrd> Range<X> {
    pub fn new(low: X, high: X) -> Range<X> {
        assert!(low < high, "Range::new called with `low >= high`");
        SampleRange::construct_range(low, high)
    }
}

impl<Sup: SampleRange> Sample<Sup> for Range<Sup> {
    #[inline]
    fn sample<R: Rng>(&mut self, rng: &mut R) -> Sup { self.ind_sample(rng) }
}
impl<Sup: SampleRange> IndependentSample<Sup> for Range<Sup> {
    fn ind_sample<R: Rng>(&self, rng: &mut R) -> Sup {
        SampleRange::sample_range(self, rng)
    }
}

pub trait SampleRange : Sized {
    fn construct_range(low: Self, high: Self) -> Range<Self>;
    fn sample_range<R: Rng>(r: &Range<Self>, rng: &mut R) -> Self;
}

impl<'a, R: ?Sized> Rng for &'a mut R where R: Rng {
    fn next_u32(&mut self) -> u32 {
        (**self).next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        (**self).next_u64()
    }

    fn next_f32(&mut self) -> f32 {
        (**self).next_f32()
    }

    fn next_f64(&mut self) -> f64 {
        (**self).next_f64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        (**self).fill_bytes(dest)
    }
}

impl<R: ?Sized> Rng for Box<R> where R: Rng {
    fn next_u32(&mut self) -> u32 {
        (**self).next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        (**self).next_u64()
    }

    fn next_f32(&mut self) -> f32 {
        (**self).next_f32()
    }

    fn next_f64(&mut self) -> f64 {
        (**self).next_f64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        (**self).fill_bytes(dest)
    }
}

#[derive(Debug)]
pub struct Generator<'a, T, R:'a> {
    rng: &'a mut R,
    _marker: marker::PhantomData<fn() -> T>,
}

impl<'a, T: Rand, R: Rng> Iterator for Generator<'a, T, R> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        Some(self.rng.gen())
    }
}

#[derive(Debug)]
pub struct AsciiGenerator<'a, R:'a> {
    rng: &'a mut R,
}

impl<'a, R: Rng> Iterator for AsciiGenerator<'a, R> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        const GEN_ASCII_STR_CHARSET: &'static [u8] =
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
              abcdefghijklmnopqrstuvwxyz\
              0123456789";
        Some(*self.rng.choose(GEN_ASCII_STR_CHARSET).unwrap() as char)
    }
}

pub trait SeedableRng<Seed>: Rng {
    fn reseed(&mut self, seed: Seed);

    fn from_seed(seed: Seed) -> Self;
}

#[allow(missing_copy_implementations)]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct XorShiftRng {
    x: w32,
    y: w32,
    z: w32,
    w: w32,
}

impl XorShiftRng {
    pub fn new_unseeded() -> XorShiftRng {
        XorShiftRng {
            x: w(0x193a6754),
            y: w(0xa8a7d469),
            z: w(0x97830e05),
            w: w(0x113ba7bb),
        }
    }
}

impl Rng for XorShiftRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }
}

impl SeedableRng<[u32; 4]> for XorShiftRng {
    fn reseed(&mut self, seed: [u32; 4]) {
        assert!(!seed.iter().all(|&x| x == 0),
                "XorShiftRng.reseed called with an all zero seed.");

        self.x = w(seed[0]);
        self.y = w(seed[1]);
        self.z = w(seed[2]);
        self.w = w(seed[3]);
    }

    fn from_seed(seed: [u32; 4]) -> XorShiftRng {
        assert!(!seed.iter().all(|&x| x == 0),
                "XorShiftRng::from_seed called with an all zero seed.");

        XorShiftRng {
            x: w(seed[0]),
            y: w(seed[1]),
            z: w(seed[2]),
            w: w(seed[3]),
        }
    }
}

impl Rand for XorShiftRng {
    fn rand<R: Rng>(rng: &mut R) -> XorShiftRng {
        let mut tuple: (u32, u32, u32, u32) = rng.gen();
        while tuple == (0, 0, 0, 0) {
            tuple = rng.gen();
        }
        let (x, y, z, w_) = tuple;
        XorShiftRng { x: w(x), y: w(y), z: w(z), w: w(w_) }
    }
}

fn weak_seed() -> [usize; 2] {
    let now = time::SystemTime::now();
    let unix_time = now.duration_since(time::UNIX_EPOCH).unwrap();
    let seconds = unix_time.as_secs() as usize;
    let nanoseconds = unix_time.subsec_nanos() as usize;
    [seconds, nanoseconds]
}

pub fn sample<T, I, R>(rng: &mut R, iterable: I, amount: usize) -> Vec<T>
    where I: IntoIterator<Item=T>,
          R: Rng,
{
    let mut iter = iterable.into_iter();
    let mut reservoir: Vec<T> = iter.by_ref().take(amount).collect();
    if reservoir.len() == amount {
        for (i, elem) in iter.enumerate() {
            let k = rng.gen_range(0, i + 1 + amount);
            if let Some(spot) = reservoir.get_mut(k) {
                *spot = elem;
            }
        }
    }
    reservoir
}

impl Rand for isize {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> isize {
        if mem::size_of::<isize>() == 4 {
            rng.gen::<i32>() as isize
        } else {
            rng.gen::<i64>() as isize
        }
    }
}

impl Rand for i8 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> i8 {
        rng.next_u32() as i8
    }
}

impl Rand for i16 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> i16 {
        rng.next_u32() as i16
    }
}

impl Rand for i32 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> i32 {
        rng.next_u32() as i32
    }
}

impl Rand for i64 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> i64 {
        rng.next_u64() as i64
    }
}

#[cfg(feature = "i128_support")]
impl Rand for i128 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> i128 {
        rng.gen::<u128>() as i128
    }
}

impl Rand for usize {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> usize {
        if mem::size_of::<usize>() == 4 {
            rng.gen::<u32>() as usize
        } else {
            rng.gen::<u64>() as usize
        }
    }
}

impl Rand for u8 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> u8 {
        rng.next_u32() as u8
    }
}

impl Rand for u16 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> u16 {
        rng.next_u32() as u16
    }
}

impl Rand for u32 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> u32 {
        rng.next_u32()
    }
}

impl Rand for u64 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> u64 {
        rng.next_u64()
    }
}

#[cfg(feature = "i128_support")]
impl Rand for u128 {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> u128 {
        ((rng.next_u64() as u128) << 64) | (rng.next_u64() as u128)
    }
}

macro_rules! integer_impl {
    ($ty:ty, $unsigned:ident) => {
        impl SampleRange for $ty {

            #[inline]
            fn construct_range(low: $ty, high: $ty) -> Range<$ty> {
                let range = (w(high as $unsigned) - w(low as $unsigned)).0;
                let unsigned_max: $unsigned = ::std::$unsigned::MAX;

                let zone = unsigned_max - unsigned_max % range;

                Range {
                    low: low,
                    range: range as $ty,
                    accept_zone: zone as $ty
                }
            }

            #[inline]
            fn sample_range<R: Rng>(r: &Range<$ty>, rng: &mut R) -> $ty {
                loop {
                    let v = rng.gen::<$unsigned>();
                    if v < r.accept_zone as $unsigned {
                        return (w(r.low) + w((v % r.range as $unsigned) as $ty)).0;
                    }
                }
            }
        }
    }
}

integer_impl! { i8, u8 }
integer_impl! { i16, u16 }
integer_impl! { i32, u32 }
integer_impl! { i64, u64 }
integer_impl! { isize, usize }
integer_impl! { u8, u8 }
integer_impl! { u16, u16 }
integer_impl! { u32, u32 }
integer_impl! { u64, u64 }
integer_impl! { usize, usize }

macro_rules! float_impl {
    ($ty:ty) => {
        impl SampleRange for $ty {
            fn construct_range(low: $ty, high: $ty) -> Range<$ty> {
                Range {
                    low: low,
                    range: high - low,
                    accept_zone: 0.0 // unused
                }
            }
            fn sample_range<R: Rng>(r: &Range<$ty>, rng: &mut R) -> $ty {
                r.low + r.range * rng.gen::<$ty>()
            }
        }
    }
}

float_impl! { f32 }
float_impl! { f64 }

macro_rules! float_impls {
    ($mod_name:ident, $ty:ty, $mantissa_bits:expr, $method_name:ident) => {
        mod $mod_name {
            use super::{Rand, Rng};
            const SCALE: $ty = (1u64 << $mantissa_bits) as $ty;

            impl Rand for $ty {
                #[inline]
                fn rand<R: Rng>(rng: &mut R) -> $ty {
                    rng.$method_name()
                }
            }
        }
    }
}
float_impls! { f64_rand_impls, f64, 53, next_f64 }
float_impls! { f32_rand_impls, f32, 24, next_f32 }

impl Rand for char {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> char {
        const CHAR_MASK: u32 = 0x001f_ffff;
        loop {
            match char::from_u32(rng.next_u32() & CHAR_MASK) {
                Some(c) => return c,
                None => {}
            }
        }
    }
}

impl Rand for bool {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> bool {
        rng.gen::<u8>() & 1 == 1
    }
}

macro_rules! tuple_impl {
    ($($tyvar:ident),* ) => {
        impl<
            $( $tyvar : Rand ),*
            > Rand for ( $( $tyvar ),* , ) {

            #[inline]
            fn rand<R: Rng>(_rng: &mut R) -> ( $( $tyvar ),* , ) {
                (
                    $(
                        _rng.gen::<$tyvar>()
                    ),*
                    ,
                )
            }
        }
    }
}

impl Rand for () {
    #[inline]
    fn rand<R: Rng>(_: &mut R) -> () { () }
}
tuple_impl!{A}
tuple_impl!{A, B}
tuple_impl!{A, B, C}
tuple_impl!{A, B, C, D}
tuple_impl!{A, B, C, D, E}
tuple_impl!{A, B, C, D, E, F}
tuple_impl!{A, B, C, D, E, F, G}
tuple_impl!{A, B, C, D, E, F, G, H}
tuple_impl!{A, B, C, D, E, F, G, H, I}
tuple_impl!{A, B, C, D, E, F, G, H, I, J}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K, L}

macro_rules! array_impl {
    {$n:expr, $t:ident, $($ts:ident,)*} => {
        array_impl!{($n - 1), $($ts,)*}

        impl<T> Rand for [T; $n] where T: Rand {
            #[inline]
            fn rand<R: Rng>(_rng: &mut R) -> [T; $n] {
                [_rng.gen::<$t>(), $(_rng.gen::<$ts>()),*]
            }
        }
    };
    {$n:expr,} => {
        impl<T> Rand for [T; $n] {
            fn rand<R: Rng>(_rng: &mut R) -> [T; $n] { [] }
        }
    };
}

array_impl!{32, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T,}

impl<T:Rand> Rand for Option<T> {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> Option<T> {
        if rng.gen() {
            Some(rng.gen())
        } else {
            None
        }
    }
}
