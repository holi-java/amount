use crate::{
    traits::{Error, Exchanger, ExchangerExt},
    Unit, UnitRate,
};

#[derive(Clone)]
pub struct Weight;

impl Exchanger for Weight {
    type Rate = u32;
    type Err = Error;
    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
        match &*unit.key {
            "t" => Ok(1_000_000),
            "kg" => Ok(1_000),
            "jin" => Ok(500),
            "g" => Ok(1),
            _ => Err(Error::NotFound(unit.clone())),
        }
    }

    fn units(&self) -> &[UnitRate<Self::Rate>] {
        use lazy_static::lazy_static;
        lazy_static! {
            static ref UNITS: [UnitRate<u32>; 4] = [
                (Unit::new("t"), 1_000_000),
                (Unit::new("kg"), 1_000),
                (Unit::new("jin"), 500),
                (Unit::new("g"), 1),
            ];
        }
        &UNITS[..]
    }
}

impl ExchangerExt for Weight {
    fn base_unit(&self) -> Unit {
        Unit::new("g")
    }
}
