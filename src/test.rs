use super::{Exchanger, Unit};
pub struct Weight;

impl Exchanger for Weight {
    type Rate = u32;
    type Err = ();
    fn rate(&self, source: &Unit, dest: &Unit) -> Self::Output {
        match (&*source.key, &*dest.key) {
            ("kg", "g") => Ok(1000),
            _ => Err(()),
        }
    }

    fn base_unit(&self) -> Unit {
        Unit::new("g")
    }
}
