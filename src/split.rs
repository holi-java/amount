use crate::{Amount, ExchangerExt, Number, Reduce, Sum};
use std::{
    fmt::Display,
    ops::{Add, Mul},
    slice::Iter,
};

#[derive(Debug, Clone)]
pub struct Split {
    pub(crate) pieces: Vec<Amount>,
}

impl Display for Split {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut pieces = self.pieces.iter();
        if let Some(it) = pieces.next() {
            Display::fmt(it, f)?;
            for it in pieces {
                f.write_str(" + ")?;
                Display::fmt(it, f)?;
            }
        }
        Ok(())
    }
}

impl Split {
    pub fn iter(&self) -> Iter<Amount> {
        self.pieces.iter()
    }
}

impl IntoIterator for Split {
    type Item = Amount;

    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.pieces.into_iter()
    }
}

impl<E: ExchangerExt> Reduce<E> for Split {
    type Output = Amount;

    fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err> {
        let mut pieces = self.pieces.iter().rev();
        let mut result = pieces.next().unwrap().reduce(&exchanger)?;
        for rest in pieces {
            let rest = rest.reduce(&exchanger)?;
            result = crate::sum2(result.amount, rest.amount, result.unit);
        }
        Ok(result)
    }
}

impl_addop!(Split => Split, Split => Amount, Split => Sum<L, R>);
impl_mulop!(
    impl<T> Mul<T> => Split |self, multiplier| {
        for amount in self.pieces.iter_mut() {
            *amount = (&*amount).mul(multiplier.clone());
        }
        self
    }
    where T: Clone, Number: Mul<T, Output = Number>,
);
