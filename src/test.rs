use super::{Exchanger, Unit};

#[derive(Clone, Copy)]
pub struct Weight;

impl Exchanger for Weight {
    type Rate = u32;
    type Err = String;
    fn rate(&self, source: &Unit, dest: &Unit) -> Self::Output {
        match (&*source.key, &*dest.key) {
            ("kg", "g") => Ok(1000),
            _ => Err(format!("can not exchange {} => {}", source.key, dest.key)),
        }
    }

    fn base_unit(&self) -> Unit {
        Unit::new("g")
    }
}
