//! A test-owned complex coefficient with only the explicitly exercised abilities.
#![allow(dead_code)]
use core::ops::{Add, Div, Mul, Neg, Sub};
use num_traits::{One, Zero};
use qmb_core::{Conjugate, Divide, ImaginaryUnit, RealPart};

#[derive(Clone, Debug, PartialEq)]
pub struct MinimalComplex<T> {
    pub re: T,
    pub im: T,
}
impl<T> MinimalComplex<T> {
    pub fn new(re: T, im: T) -> Self {
        Self { re, im }
    }
}
impl<T: Add<Output = T>> Add for MinimalComplex<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}
impl<T: Clone + Add<Output = T> + Sub<Output = T> + Mul<Output = T>> Mul for MinimalComplex<T> {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(
            self.re.clone() * rhs.re.clone() - self.im.clone() * rhs.im.clone(),
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}
impl<T: Neg<Output = T>> Neg for MinimalComplex<T> {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.re, -self.im)
    }
}
impl<T: Zero> Zero for MinimalComplex<T> {
    fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }
    fn is_zero(&self) -> bool {
        self.re.is_zero() && self.im.is_zero()
    }
}
impl<T: Clone + Zero + One + Sub<Output = T>> One for MinimalComplex<T> {
    fn one() -> Self {
        Self::new(T::one(), T::zero())
    }
}
impl<T: Clone + Neg<Output = T>> Conjugate for MinimalComplex<T> {
    type Output = Self;
    fn conjugate(&self) -> Self {
        Self::new(self.re.clone(), -self.im.clone())
    }
}
impl<T> RealPart for MinimalComplex<T> {
    type Real = T;
    fn real_part(self) -> T {
        self.re
    }
}
impl<T: Zero + One> ImaginaryUnit for MinimalComplex<T> {
    fn imaginary_unit() -> Self {
        Self::new(T::zero(), T::one())
    }
}
impl<T: Clone + Zero + Sub<Output = T> + Mul<Output = T> + Div<Output = T>> Divide
    for MinimalComplex<T>
{
    type Output = Self;
    fn divide(self, rhs: Self) -> Self {
        // Test-owned coefficients keep their unordered arithmetic contract;
        // they need not use the standard Complex floating-point algorithm.
        if rhs.im.is_zero() {
            return Self::new(self.re / rhs.re.clone(), self.im / rhs.re);
        }
        let norm = rhs.re.clone() * rhs.re.clone() + rhs.im.clone() * rhs.im.clone();
        Self::new(
            (self.re.clone() * rhs.re.clone() + self.im.clone() * rhs.im.clone()) / norm.clone(),
            (self.im * rhs.re - self.re * rhs.im) / norm,
        )
    }
}
