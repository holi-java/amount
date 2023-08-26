use crate::{Amount, Number, Split, Unit};
pub trait Exchanger {
    type Rate: Into<Number>;
    type Err;

    fn rate(&self, unit: &Unit) -> Result<Self::Rate, Self::Err>;
}

/// Each product has its own [Exchanger] for reduce [Amount] with diff [Unit]s
/// into single [Amount] with [base unit](Self::base_unit()).
pub trait ExchangerExt: Exchanger {
    fn base_unit(&self) -> Unit;

    fn sorted_units(&self) -> &[Unit];

    fn reduce<T>(&self, exp: T) -> Result<Amount, Self::Err>
    where
        for<'a> T: Reduce<&'a Self, Output = Amount>,
        Self: Sized,
    {
        exp.reduce(self)
    }

    fn split<T>(&self, exp: T) -> Result<Split, Self::Err>
    where
        for<'a> T: Reduce<&'a Self, Output = Amount>,
        Self: Sized,
    {
        let Amount {
            amount: mut remaining,
            unit: _,
        } = self.reduce(exp)?;

        let mut pieces = vec![];
        for current in self.sorted_units() {
            let rate: Number = self.rate(current)?.into();
            if remaining >= rate {
                pieces.push(Amount::new(remaining / rate, current.clone()));
                remaining %= rate;
            }
        }
        Ok(Split { pieces })
    }
}

impl_exchanger!(&T);
impl_exchanger!(&mut T);

pub trait Reduce<E: Exchanger> {
    type Output;

    fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err>;
}

impl_reduce!(&T);
impl_reduce!(&mut T);
