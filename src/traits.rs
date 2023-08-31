use std::fmt::Display;

use crate::{split::Split, table::Table, Amount, Number, Unit};

pub(crate) type UnitRate<T> = (Unit, T);

/// Each product has its own [Exchanger] for reduce [Amount] with diff [Unit]s
/// into single [Amount] with [base unit](Exchanger::base_unit()).
pub trait Exchanger {
    type Rate: Into<Number> + Ord + Clone;
    type Err;

    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err>;

    fn sorted_units(&self) -> &[UnitRate<Self::Rate>];

    fn base_unit(&self) -> &Unit;
}

pub trait ExchangerExt: Exchanger {
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
        for (current, rate) in self.sorted_units() {
            let rate: Number = rate.clone().into();
            if remaining >= rate {
                pieces.push(Amount::new(remaining / rate, current.clone()));
                remaining %= rate;
            }
        }
        Ok(Split { pieces })
    }

    fn extend<E>(self, ext: E) -> Result<Table, Error>
    where
        E: Exchanger,
        Number: From<Self::Rate>,
        Number: From<E::Rate>,
        Error: From<E::Err>,
        Error: From<Self::Err>,
        Self: Sized,
    {
        crate::table::merge::<Self, E>(self, ext)
    }
}

impl<T: Exchanger> ExchangerExt for T {}

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
    use std::ops::Add;

    use super::{Exchanger, ExchangerExt};
    use crate::{
        test::{bag, g, kg, CustomWeight},
        Amount, Weight,
    };

    #[test]
    fn extend_exchanger() {
        let ext = Weight::default().extend(CustomWeight::default()).unwrap();

        assert_eq!(ext.rate(&kg()).unwrap(), 1_000);
        assert_eq!(ext.rate(&bag()).unwrap(), 45_000);
    }

    #[test]
    fn add_empty_split() {
        let ext = Weight::default().extend(CustomWeight::default()).unwrap();
        let empty = ext.split(Amount::new(0, kg())).unwrap();

        assert_eq!(empty.iter().cloned().collect::<Vec<_>>(), []);

        let sum = empty.add(Amount::new(1_100, g()));
        let split = ext.split(sum).unwrap();
        assert_eq!(
            split.iter().cloned().collect::<Vec<_>>(),
            [Amount::new(1, kg()), Amount::new(100, g())]
        );
    }
}
