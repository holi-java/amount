#![allow(unused)]
use std::{
    convert::Infallible,
    fmt::Display,
    ops::{FromResidual, Try},
};

use crate::{traits::Error, Exchanger, ExchangerExt, Number, Unit};

pub struct Extend<B, E> {
    ext: E,
    base: B,
    units: Vec<Unit>,
}

pub(crate) fn extend<B, E>(base: B, ext: E) -> Extend<B, E>
where
    B: Exchanger,
    E: Exchanger,
    Error: From<E::Err>,
    Error: From<B::Err>,
    Number: From<E::Rate>,
    Number: From<B::Rate>,
{
    fn sorted_unit_rates_of<T: Exchanger>(exp: &T) -> impl Iterator<Item = (Unit, Number)> + '_
    where
        Number: From<T::Rate>,
    {
        exp.sorted_units().iter().map(|unit| {
            (
                unit.clone(),
                exp.rate(unit)
                    .map_err(|_| ())
                    .expect("violate Exchanger contract")
                    .into(),
            )
        })
    }
    Extend {
        units: {
            let mut units = sorted_unit_rates_of(&ext).collect::<Vec<_>>();
            units.extend(sorted_unit_rates_of(&base));
            units.sort_by_key(|&(_, rate)| rate);
            units.into_iter().rev().map(|(unit, _)| unit).collect()
        },
        ext,
        base,
    }
}

impl<E, B, E1, E2> Exchanger for Extend<B, E>
where
    E: Exchanger<Err = E1>,
    B: Exchanger<Err = E2>,
    Number: From<E::Rate>,
    Number: From<B::Rate>,
    Error: From<E1>,
    Error: From<E2>,
{
    type Rate = Number;

    type Err = Error;

    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
        match self.ext.rate(unit) {
            Ok(rate) => Ok(rate.into()),
            _ => Ok(self.base.rate(unit)?.into()),
        }
    }

    fn sorted_units(&self) -> &[Unit] {
        &self.units
    }
}

impl<E, B, E1, E2> ExchangerExt for Extend<B, E>
where
    E: Exchanger<Err = E1>,
    B: Exchanger<Err = E2>,
    Number: From<E::Rate>,
    Number: From<B::Rate>,
    Error: From<E1>,
    Error: From<E2>,
    B: ExchangerExt,
{
    fn base_unit(&self) -> Unit {
        self.base.base_unit()
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;

    use super::{extend, Error};
    use crate::{test::*, Amount};
    use crate::{Exchanger, ExchangerExt, Number, Unit, Weight};

    #[test]
    fn compose_two_exchangers_units() {
        let ext = extend(Weight, CustomWeight);
        assert_eq!(
            ext.sorted_units(),
            [
                Unit::new("t"),
                Unit::new("bag"),
                Unit::new("kg"),
                Unit::new("jin"),
                Unit::new("g")
            ]
        );
    }

    #[test]
    fn compose_two_exchangers_rates() {
        let ext = extend(Weight, CustomWeight);

        assert_eq!(ext.rate(&kg()).unwrap(), Weight.rate(&kg()).unwrap().into());
        assert_eq!(
            ext.rate(&bag()).unwrap(),
            CustomWeight.rate(&bag()).unwrap().into()
        );
    }

    #[test]
    fn reduce_on_composed_exchanger() {
        let ext = extend(Weight, CustomWeight);
        assert_eq!(
            ext.reduce(Amount::new(1, bag())).unwrap(),
            Amount::new(45_000, g())
        );
        assert_eq!(
            ext.reduce(Amount::new(1, kg())).unwrap(),
            Amount::new(1_000, g())
        );
    }
}
