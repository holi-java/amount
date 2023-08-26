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

    fn sorted_units(&self) -> &[Unit] {
        use lazy_static::lazy_static;
        lazy_static! {
            static ref UNITS: [Unit; 1] = [Unit::new("bag")];
        }
        &UNITS[..]
    }
}
