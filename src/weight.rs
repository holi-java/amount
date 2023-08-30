use crate::{Error, ExchangerExt, Number, Table, UnitRate};

define_exchanger!(
    #[base_unit = "g"]
    #[derive(Clone)]
    pub Weight {
        t = 1_000_000,
        kg = 1_000,
        jin = 500
    }
);

pub fn custom<I, R>(base_unit: &str, units: I) -> Result<Table, Error>
where
    I: IntoIterator<Item = UnitRate<R>>,
    Number: From<R>,
{
    let table = Table::new(base_unit, units);
    if Weight::units().any(|unit| unit == base_unit) {
        Weight::base(base_unit)?.extend::<Table>(table)
    } else {
        Ok(table)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Exchanger, Unit};

    #[test]
    fn create_custom_with_weight_base_unit() {
        let weight = custom(
            "g",
            [
                (Unit::new("bag"), 45_000_u32),
                (Unit::new("box"), 100_000_u32),
            ],
        )
        .unwrap();

        assert_eq!(
            weight.sorted_units().to_vec(),
            [
                (Unit::new("t"), 1_000_000),
                (Unit::new("box"), 100_000),
                (Unit::new("bag"), 45_000),
                (Unit::new("kg"), 1_000),
                (Unit::new("jin"), 500),
                (Unit::new("g"), 1),
            ]
        );
    }

    #[test]
    fn create_custom_with_weight_high_unit() {
        let weight = custom(
            "kg",
            [(Unit::new("bag"), 45_u32), (Unit::new("box"), 100_u32)],
        )
        .unwrap();

        assert_eq!(
            weight.sorted_units().to_vec(),
            [
                (Unit::new("t"), 1_000),
                (Unit::new("box"), 100),
                (Unit::new("bag"), 45),
                (Unit::new("kg"), 1),
            ]
        );
    }

    #[test]
    fn create_custom_weight_without_base_unit() {
        let weight = custom(
            "bag",
            [(Unit::new("car"), 500_u32), (Unit::new("box"), 20_u32)],
        )
        .unwrap();

        assert_eq!(
            weight.sorted_units().to_vec(),
            [
                (Unit::new("car"), 500),
                (Unit::new("box"), 20),
                (Unit::new("bag"), 1),
            ]
        );
    }
}
