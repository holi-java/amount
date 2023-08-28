use crate::define_exchanger;

use super::Unit;

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

define_exchanger!(
    #[base_unit = "kg"]
    pub CustomWeight {
        bag = 45
    }
);
