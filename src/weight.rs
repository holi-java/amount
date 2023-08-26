use crate::{Exchanger, ExchangerExt, Unit};

#[derive(Clone, Copy)]
pub struct Weight;

impl Exchanger for Weight {
    type Rate = u32;
    type Err = String;
    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
        match &*unit.key {
            "t" => Ok(1_000_000),
            "kg" => Ok(1_000),
            "jin" => Ok(500),
            "g" => Ok(1),
            _ => Err(format!("can not exchange {}", unit.key)),
        }
    }
}

impl ExchangerExt for Weight {
    fn base_unit(&self) -> Unit {
        Unit::new("g")
    }

    fn sorted_units(&self) -> &[Unit] {
        use lazy_static::lazy_static;
        lazy_static! {
            static ref UNITS: [Unit; 4] = [
                Unit::new("t"),
                Unit::new("kg"),
                Unit::new("jin"),
                Unit::new("g")
            ];
        }
        &UNITS[..]
    }
}
