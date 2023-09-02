use crate::{traits::Error, Exchanger, Number, Unit, UnitRate};

#[derive(Debug, Clone)]
pub struct Table {
    units: Vec<UnitRate<Number>>,
    base_unit: Unit,
}

impl Table {
    pub fn new<U: Into<Unit>, I, R>(base_unit: U, units: I) -> Self
    where
        I: IntoIterator<Item = UnitRate<R>>,
        Number: From<R>,
    {
        let base_unit = base_unit.into();
        let iter = units.into_iter();
        let mut units: Vec<UnitRate<Number>> = iter
            .size_hint()
            .1
            .map(Vec::with_capacity)
            .unwrap_or_default();
        let mut has_base_unit = false;
        for (unit, rate) in iter {
            if !has_base_unit && unit.key == base_unit.key {
                has_base_unit = true;
            }
            units.push((unit, rate.into()));
        }
        units.sort_by(|(_, a), (_, b)| a.cmp(b).reverse());

        if !has_base_unit {
            units.push((base_unit.clone(), 1));
        }

        Table { base_unit, units }
    }
}

pub(crate) fn merge<B, E>(base: B, ext: E) -> Result<Table, Error>
where
    B: Exchanger,
    E: Exchanger,
    Error: From<E::Err>,
    Error: From<B::Err>,
    Number: From<E::Rate>,
    Number: From<B::Rate>,
{
    let merge = ext.base_unit();
    let units = scaled(
        ext.sorted_units().iter().filter(|(unit, _)| unit != merge),
        base.rate(merge).map(Number::from)?,
    );
    let units = units.chain(scaled(base.sorted_units(), 1));
    return Ok(Table::new(base.base_unit(), units));

    #[inline]
    #[cold]
    fn scaled<'a, I, R: 'a>(e: I, diff: Number) -> impl Iterator<Item = UnitRate<Number>>
    where
        I: IntoIterator<Item = &'a UnitRate<R>>,
        Number: From<R>,
        R: Clone,
    {
        e.into_iter()
            .cloned()
            .map(move |(unit, rate)| (unit.clone(), Number::from(rate) * diff))
    }
}

impl Exchanger for Table {
    type Rate = Number;

    type Err = Error;

    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
        for (test, rate) in &self.units {
            if test == unit {
                return Ok(*rate);
            }
        }
        Err(Error::NotFound(unit.clone()))
    }

    fn sorted_units(&self) -> &[UnitRate<Self::Rate>] {
        &self.units
    }

    fn base_unit(&self) -> &Unit {
        &self.base_unit
    }
}

#[cfg(test)]
mod tests {
    use super::merge;
    use crate::test::*;
    use crate::{Amount, Exchanger, ExchangerExt, Unit, Weight};

    #[test]
    fn compose_two_exchangers_units() {
        let ext = merge(Weight::default(), CustomWeight::default()).unwrap();
        assert_eq!(
            ext.sorted_units()
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
        let ext = merge(Weight::default(), CustomWeight::default()).unwrap();

        assert_eq!(
            ext.rate(&kg()).unwrap(),
            Weight::default().rate(&kg()).unwrap()
        );
        assert_eq!(
            ext.rate(&bag()).unwrap(),
            CustomWeight::default().rate(&bag()).unwrap() * Weight::default().rate(&kg()).unwrap()
        );
    }

    #[test]
    fn reduce_on_composed_exchanger() {
        let ext = merge(Weight::default(), CustomWeight::default()).unwrap();
        assert_eq!(
            ext.reduce(Amount::new(1, bag())).unwrap(),
            Amount::new(45_000, g())
        );
        assert_eq!(
            ext.reduce(Amount::new(1, kg())).unwrap(),
            Amount::new(1_000, g())
        );
    }

    #[test]
    fn extend_exchangers_with_diff_base_units() {
        define_exchanger!(#[base_unit = "kg"] CustomWeight {
            box = 2_000,
            bag = 50
        });

        let ext = merge(Weight::base("jin").unwrap(), CustomWeight::default()).unwrap();

        assert_eq!(
            ext.sorted_units(),
            [
                (Unit::new("box"), 4_000),
                (Unit::new("t"), 2_000),
                (Unit::new("bag"), 100),
                (Unit::new("kg"), 2),
                (Unit::new("jin"), 1),
            ]
        );
        assert_eq!(
            ext.reduce(Amount::new(1, Unit::new("box"))).unwrap(),
            Amount::new(4_000, Unit::new("jin"))
        );
    }

    use super::Table;
    #[test]
    fn new_table_unit_rates_contains_base_unit() {
        let table = Table::new("g", [(Unit::new("kg"), 1000_u64), (Unit::new("g"), 1)]);

        assert_eq!(
            table
                .sorted_units()
                .iter()
                .map(|(unit, _)| unit.key.clone())
                .collect::<Vec<_>>(),
            ["kg", "g"]
        );
        assert_eq!(table.base_unit().key, "g");
    }

    #[test]
    fn add_base_unit_when_new_table_unit_rates_does_not_contains_base_unit() {
        let table = Table::new("g", [(Unit::new("kg"), 1000_u64)]);

        assert_eq!(
            table.sorted_units().to_vec(),
            [(Unit::new("kg"), 1_000), (Unit::new("g"), 1),]
        );
        assert_eq!(table.base_unit().key, "g");
    }
}