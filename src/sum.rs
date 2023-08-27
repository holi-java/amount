use crate::{split::Split, Amount, Exchanger, Reduce};
use std::{
    fmt::Display,
    ops::{Add, Mul},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Sum<L, R>(pub(crate) L, pub(crate) R);

impl<L: Display, R: Display> Display for Sum<L, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{lhs} + {rhs}", lhs = self.0, rhs = self.1)
    }
}

impl_addop!(Sum<L, R> => Split, Sum<L, R> => Amount, Sum<L, R> => Sum<Q, P>);
impl_mulop!(
    <T, L, R> Mul<T> => Sum<L, R> : [T: Clone, L: Mul<T, Output = L> + Clone, R: Mul<T, Output = R> + Clone] {
        (self, multiplier) -> {
                Sum(self.0 * multiplier.clone(), self.1 * multiplier)
        }
    }
);

impl<L, R, E> Reduce<E> for Sum<L, R>
where
    L: Reduce<E, Output = Amount>,
    R: Reduce<E, Output = Amount>,
    E: Exchanger + Clone,
{
    type Output = Amount;

    fn reduce(&self, exchanger: E) -> Result<Self::Output, E::Err> {
        let (lhs, rhs) = (self.0.reduce(exchanger.clone())?, self.1.reduce(exchanger)?);
        Ok(crate::sum2(lhs.amount, rhs.amount, lhs.unit))
    }
}
