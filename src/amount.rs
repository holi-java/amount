use std::{
    fmt::Display,
    num::ParseIntError,
    ops::{Add, Mul},
    str::FromStr,
};

pub(crate) type Number = u64;
use crate::{split::Split, sum::Sum, Exchanger, Reduce};

#[derive(Debug, Clone, PartialEq)]
pub struct Amount {
    pub amount: Number,
    pub unit: Unit,
}

impl Amount {
    pub fn new(amount: Number, unit: Unit) -> Self {
        Amount { amount, unit }
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{amount}{unit}", amount = self.amount, unit = self.unit)
    }
}

#[derive(Debug)]
pub enum ParseError {
    Empty,
    ParseError(ParseIntError),
}

impl std::error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("Empty"),
            Self::ParseError(err) => Display::fmt(err, f),
        }
    }
}

impl FromStr for Amount {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let i = s
            .find(|c: char| !c.is_ascii_digit())
            .ok_or(ParseError::Empty)?;
        Ok(Amount::new(
            s[..i].parse().map_err(ParseError::ParseError)?,
            Unit::new(&s[i..]),
        ))
    }
}

impl_addop!(Amount => Split, Amount => Amount, Amount => Sum<L, R>);
impl_mulop!(
    <T> Mul<T> => Amount: [Number: Mul<T, Output = Number>] {
        (self, multiplier) -> {
            Amount::new(self.amount * multiplier, self.unit)
        }
    }
);

impl<E: Exchanger> Reduce<E> for Amount {
    type Output = Amount;

    fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err> {
        Ok(Amount::new(
            // TODO: overflow
            self.amount * exchanger.rate(&self.unit)?.into(),
            exchanger.base_unit().clone(),
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

impl<T: Into<String>> From<T> for Unit {
    fn from(unit: T) -> Self {
        Self::new(unit)
    }
}

impl From<&Unit> for Unit {
    fn from(unit: &Unit) -> Self {
        unit.clone()
    }
}

pub(crate) fn sum2(lhs: Number, rhs: Number, unit: crate::Unit) -> Amount {
    // TODO: overflow
    Amount::new(lhs + rhs, unit)
}
