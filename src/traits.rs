use std::fmt::Display;

use crate::{extend::Extend, split::Split, Amount, Number, Unit};
pub trait Exchanger {
    type Rate: Into<Number>;
    type Err;

    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err>;

    fn sorted_units(&self) -> &[Unit];
}

/// Each product has its own [Exchanger] for reduce [Amount] with diff [Unit]s
/// into single [Amount] with [base unit](Self::base_unit()).
pub trait ExchangerExt: Exchanger {
    fn base_unit(&self) -> Unit;

    fn reduce<T>(&self, exp: T) -> Result<Amount, Self::Err>
    where
        for<'a> T: Reduce<&'a Self, Output = Amount>,
        Self: Sized,
    {
        exp.reduce(self)
    }

    fn split<T>(&self, exp: T) -> Result<Split, Self::Err>
    where
        for<'a> T: Reduce<&'a Self, Output = Amount>,
        Self: Sized,
    {
        let Amount {
            amount: mut remaining,
            unit: _,
        } = self.reduce(exp)?;

        let mut pieces = vec![];
        for current in self.sorted_units() {
            let rate: Number = self.rate(current)?.into();
            if remaining >= rate {
                pieces.push(Amount::new(remaining / rate, current.clone()));
                remaining %= rate;
            }
        }
        Ok(Split { pieces })
    }

    fn extend<E>(self, ext: E) -> Extend<Self, E>
    where
        E: Exchanger,
        Number: From<Self::Rate>,
        Number: From<E::Rate>,
        Error: From<E::Err>,
        Error: From<Self::Err>,
        Self: Sized,
    {
        crate::extend::extend::<Self, E>(self, ext)
    }
}

pub trait Reduce<E: Exchanger> {
    type Output;

    fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err>;
}

parse_generic_types!(impl_all_traits!(&T, &mut T, Box<T>));

#[derive(Debug)]
pub enum Error {
    NotFound(Unit),
    Message(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(unit) => write!(f, "Unit not found: {:?}", unit.key),
            Self::Message(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Message(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{Exchanger, ExchangerExt};
    use crate::{
        test::{bag, kg, CustomWeight},
        Weight,
    };

    #[test]
    fn extend_exchanger() {
        let ext = Weight.extend(CustomWeight);

        assert_eq!(ext.rate(&kg()).unwrap(), 1_000);
        assert_eq!(ext.rate(&bag()).unwrap(), 45_000);
    }
}
