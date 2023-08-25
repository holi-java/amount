#![doc(html_no_source, html_playground_url = "https://play.rust-lang.org/")]
#![feature(try_trait_v2)]
#![feature(associated_type_defaults)]
//!
//!
//! # Example
//!
//! [Reduce] sum of the two amounts to [Amount] with base [unit](Unit).
//!
//! ```rust
//! # use crate::amount::{Amount, Unit, Exchanger};
//! # mod test;
//! # use test::Weight;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let five = "5kg".parse::<Amount>()?;
//! let two = "2g".parse::<Amount>()?;
//!
//! let sum = five + two;
//! let result = sum * 3;
//!
//! let result = Weight.reduce(result)?;
//!
//! assert_eq!(result, Amount::new(15006, Weight.base_unit()));
//! # Ok(())
//! # }
//! ```
use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use std::ops::{Add, FromResidual, Mul, Try};
use std::str::FromStr;

/// Each product has its own [Exchanger] for reduce [Amount] with diff [Unit]s 
/// into single [Amount] with [base unit](Self::base_unit()).
pub trait Exchanger {
    type Rate: Into<u32>;
    type Err;
    type Output: Try<Output = Self::Rate> = Result<Self::Rate, Self::Err>;

    fn rate(&self, source: &Unit, dest: &Unit) -> Self::Output;

    fn base_unit(&self) -> Unit;

    fn reduce<T>(&self, exp: T) -> Result<Amount, Self::Err>
    where
        for<'a> T: Reduce<&'a Self, Output = Amount>,
        Self: Sized,
    {
        exp.reduce(self, &self.base_unit())
    }
}

macro_rules! impl_exchanger {
    (&$ty:ty) => {impl_exchanger!(@&);};
    (&mut $ty:ty) => {impl_exchanger!(@&mut );};
    (@&$($mut:ident)?) => {
        impl<E: Exchanger> Exchanger for &$($mut)? E {
            type Rate = E::Rate;

            type Err = E::Err;
            type Output = E::Output;

            fn rate(&self, source: &Unit, dest: &Unit) -> Self::Output {
                (**self).rate(source, dest)
            }

            fn base_unit(&self) -> Unit {
                (**self).base_unit()
            }
        }
    };
}

impl_exchanger!(&T);
impl_exchanger!(&mut T);

pub trait Reduce<E: Exchanger> {
    type Output;

    fn reduce(&self, exchanger: E, dest: &Unit) -> Result<Self::Output, E::Err>;
}

macro_rules! impl_reduce {
    (&$ty:ty) => {impl_reduce!(@&);};
    (&mut $ty:ty) => {impl_reduce!(@&mut );};
    (@&$($mut:ident)?) => {
        impl<T: Reduce<E>, E: Exchanger> Reduce<E> for &$($mut)? T {
            type Output = T::Output;

            fn reduce(&self, exchanger: E, dest: &Unit) -> Result<Self::Output, E::Err> {
                (**self).reduce(exchanger, dest)
            }
        }
    };
}

impl_reduce!(&T);
impl_reduce!(&mut T);

#[derive(Debug, Clone, PartialEq)]
pub struct Amount {
    pub amount: u32,
    pub unit: Unit,
}

impl Amount {
    pub fn new(amount: u32, unit: Unit) -> Self {
        Amount { amount, unit }
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{amount}{unit}", amount = self.amount, unit = self.unit)
    }
}

#[derive(Debug)]
pub enum Error {
    Empty,
    ParseError(ParseIntError),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("Empty"),
            Self::ParseError(err) => Display::fmt(err, f),
        }
    }
}

impl FromStr for Amount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i = s.find(|c: char| !c.is_ascii_digit()).ok_or(Error::Empty)?;
        Ok(Amount::new(
            s[..i].parse().map_err(Error::ParseError)?,
            Unit::new(&s[i..]),
        ))
    }
}

impl<Rhs> Add<Rhs> for Amount {
    type Output = Sum<Self, Rhs>;
    fn add(self, addend: Rhs) -> Self::Output {
        Sum(self, addend)
    }
}

impl<T> Mul<T> for Amount
where
    u32: Mul<T, Output = u32>,
{
    type Output = Self;
    fn mul(self, multiplier: T) -> Self::Output {
        Amount::new(self.amount * multiplier, self.unit)
    }
}

impl<E: Exchanger> Reduce<E> for Amount
where
    Result<Amount, <E as Exchanger>::Err>: FromResidual<<E::Output as Try>::Residual>,
{
    type Output = Amount;

    fn reduce(&self, exchanger: E, dest: &Unit) -> Result<Self::Output, E::Err> {
        if self.unit == *dest {
            return Ok(self.clone());
        }
        Ok(Amount::new(
            self.amount * exchanger.rate(&self.unit, dest)?.into(),
            dest.clone(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    pub key: String,
}

impl Unit {
    pub fn new<K: Into<String>>(key: K) -> Unit {
        Unit { key: key.into() }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct Sum<L, R>(L, R);

impl<L: Display, R: Display> Display for Sum<L, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{lhs} + {rhs}", lhs = self.0, rhs = self.1)
    }
}

impl<L, R, Rhs> Add<Rhs> for Sum<L, R> {
    type Output = Sum<Self, Rhs>;

    fn add(self, addend: Rhs) -> Self::Output {
        Sum(self, addend)
    }
}

impl<L, R, E> Reduce<E> for Sum<L, R>
where
    L: Reduce<E, Output = Amount>,
    R: Reduce<E, Output = Amount>,
    E: Exchanger + Clone,
{
    type Output = Amount;

    fn reduce(&self, exchanger: E, dest: &Unit) -> Result<Self::Output, E::Err> {
        let (lhs, rhs) = (
            self.0.reduce(exchanger.clone(), dest)?,
            self.1.reduce(exchanger, dest)?,
        );
        Ok(Amount::new(lhs.amount + rhs.amount, lhs.unit))
    }
}

impl<T: Clone, L, R> Mul<T> for Sum<L, R>
where
    L: Mul<T, Output = L>,
    R: Mul<T, Output = R>,
{
    type Output = Self;
    fn mul(self, multiplier: T) -> Self::Output {
        Sum(self.0 * multiplier.clone(), self.1 * multiplier)
    }
}

#[cfg(any(test, doctest))]
pub mod test;

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test::Weight;

    fn kg() -> Unit {
        Unit::new("kg")
    }

    fn g() -> Unit {
        Unit::new("g")
    }

    #[test]
    fn unit_to_string() {
        assert_eq!(g().to_string(), "g");
        assert_eq!(kg().to_string(), "kg");
    }

    #[test]
    fn amount_to_string() {
        assert_eq!(Amount::new(1, g()).to_string(), "1g");
        assert_eq!(Amount::new(5, kg()).to_string(), "5kg");
    }

    #[test]
    fn sum_to_string() {
        let one = Amount::new(1, g());
        let five = Amount::new(5, kg());
        let sum = one.add(five);

        assert_eq!(sum.to_string(), "1g + 5kg");
    }

    #[test]
    fn add_amount_with_same_unit() {
        let one = Amount::new(1, g());
        let five = Amount::new(5, g());

        let result = one.clone().add(five.clone());
        assert_eq!(result.to_string(), "1g + 5g");
    }

    #[test]
    fn amount_multiplication() {
        let five = Amount::new(5, g());

        let result = five * 3;

        assert_eq!(result.to_string(), "15g");
    }

    #[test]
    fn sum_add_amount() {
        let one = Amount::new(1, g());
        let two = Amount::new(2, g());
        let five = Amount::new(5, kg());

        let result = one.clone().add(five.clone());
        let result = result.add(two.clone());

        assert_eq!(result.to_string(), "1g + 5kg + 2g");
    }

    #[test]
    fn add_sum2() {
        let one = Amount::new(1, g());
        let two = Amount::new(2, g());
        let five = Amount::new(5, kg());

        let sum1 = one.clone().add(five.clone());
        let sum2 = one.clone().add(two.clone());
        // compiler error: need type annotation
        let result = sum1.add(sum2);

        assert_eq!(result.to_string(), "1g + 5kg + 1g + 2g");
    }

    #[test]
    fn sum_multiplication() {
        let one = Amount::new(1, g());
        let five = Amount::new(5, kg());

        let result = one.clone().add(five.clone());
        let result = result * 3;

        assert_eq!(result.to_string(), "3g + 15kg");
    }

    #[test]
    fn reduce_amount_to_same_unit() {
        let one = Amount::new(1, g());

        let result = Weight.reduce(&one).unwrap();
        assert_eq!(result, one);
    }

    #[test]
    fn reduce_amount_to_diff_unit() {
        let one = Amount::new(1, kg());

        let result = Weight.reduce(one).unwrap();
        assert_eq!(result, Amount::new(1000, g()));
    }

    #[test]
    fn reduce_sum_to_same_unit() {
        let one = Amount::new(1, g());
        let five = Amount::new(5, g());

        let sum = one.add(five);

        let result = Weight.reduce(sum).unwrap();
        assert_eq!(result, Amount::new(6, g()));
    }

    #[test]
    fn reduce_sum_to_diff_unit() {
        let one = Amount::new(1, kg());
        let five = Amount::new(5, g());

        let sum = one.add(five);

        let result = Weight.reduce(sum).unwrap();
        assert_eq!(result, Amount::new(1005, g()));
    }

    #[test]
    fn parse_amount_from_string() {
        assert_eq!(Amount::new(1, g()), "1g".parse().unwrap());
        assert_eq!(Amount::new(2, g()), "2g".parse().unwrap());
        assert_eq!(Amount::new(12, g()), "12g".parse().unwrap());
    }
}
