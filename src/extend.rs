use crate::{traits::Error, Exchanger, ExchangerExt, Number, Unit, UnitRate};

pub struct Extend<B, E> {
    ext: E,
    base: B,
    units: Vec<UnitRate<Number>>,
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
    Extend {
        units: {
            #[inline]
            #[cold]
            fn cloned_units<E>(e: &E) -> impl Iterator<Item = UnitRate<Number>> + '_
            where
                E: Exchanger,
            {
                e.units()
                    .iter()
                    .cloned()
                    .map(|(unit, rate)| (unit, rate.into()))
            }

            let mut units: Vec<_> = cloned_units(&ext).chain(cloned_units(&base)).collect();
            units.sort_by(|(_, a), (_, b)| a.cmp(b).reverse());
            units
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

    fn units(&self) -> &[UnitRate<Self::Rate>] {
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
    use super::extend;
    use crate::test::*;
    use crate::{Amount, Exchanger, ExchangerExt, Unit, Weight};

    #[test]
    fn compose_two_exchangers_units() {
        let ext = extend(Weight, CustomWeight);
        assert_eq!(
            ext.units()
                .iter()
                .map(|(unit, _)| unit.clone())
                .collect::<Vec<_>>(),
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
