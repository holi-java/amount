use super::Unit;
use super::{Error, Exchanger};

#[inline]
#[cold]
#[allow(dead_code)]
pub fn g() -> Unit {
    Unit::new("g")
}

#[inline]
#[cold]
#[allow(dead_code)]
pub fn kg() -> Unit {
    Unit::new("kg")
}

#[inline]
#[cold]
#[allow(dead_code)]
pub fn bag() -> Unit {
    Unit::new("bag")
}

type UnitRate = (Unit, u32);

pub struct CustomWeight;
impl Exchanger for CustomWeight {
    type Rate = u32;

    type Err = Error;

    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err> {
        match &*unit.key {
            "bag" => Ok(45_000),
            _ => Err(Error::NotFound(unit.clone())),
        }
    }

    fn units(&self) -> &[UnitRate] {
        use lazy_static::lazy_static;
        lazy_static! {
            static ref UNITS: [UnitRate; 1] = [(Unit::new("bag"), 45_000)];
        }
        &UNITS[..]
    }
}
