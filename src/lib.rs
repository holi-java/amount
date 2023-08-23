#![allow(unused)]
use std::fmt::{Debug, Display};
trait Boxed {
    type Output;

    fn boxed(self) -> Self::Output;
}

impl<T> Boxed for T {
    type Output = Self;

    fn boxed(self) -> Self::Output {
        self
    }
}

trait Exchanger {
    type Err;

    fn rate(&self, source: &Unit, dest: &Unit) -> Result<u32, Self::Err>;
}

trait Product {
    type Output;

    fn times(self, multiplier: u32) -> Self::Output;
}

trait Reduce {
    type Output;

    fn reduce<E: Exchanger>(&self, exchanger: &E, dest: &Unit) -> Result<Self::Output, E::Err>;
}

trait Expression<Rhs = Self> {
    fn add(self, addend: Rhs) -> Sum<Self, Rhs>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq)]
struct Amount {
    amount: u32,
    unit: Unit,
}

impl Amount {
    fn new(amount: u32, unit: Unit) -> Self {
        Amount { amount, unit }
    }
}

impl Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{amount}{unit}", amount = self.amount, unit = self.unit)
    }
}

impl<Rhs> Expression<Rhs> for Amount {
    fn add(self, addend: Rhs) -> Sum<Self, Rhs>
    where
        Self: Sized,
    {
        Sum(self, addend)
    }
}

impl Product for Amount {
    type Output = Self;
    fn times(self, multiplier: u32) -> Self::Output {
        Amount::new(self.amount * multiplier, self.unit)
    }
}

impl Reduce for Amount {
    type Output = Amount;

    fn reduce<E: Exchanger>(&self, exchanger: &E, dest: &Unit) -> Result<Self::Output, E::Err> {
        if self.unit == *dest {
            return Ok(self.clone());
        }
        Ok(Amount::new(
            self.amount * exchanger.rate(&self.unit, dest)?,
            dest.clone(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Unit {
    key: String,
}

impl Unit {
    fn new<K: Into<String>>(key: K) -> Unit {
        Unit { key: key.into() }
    }
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.key)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Sum<L, R>(L, R);

impl<L: Display, R: Display> Display for Sum<L, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{lhs} + {rhs}", lhs = self.0, rhs = self.1)
    }
}

impl<L, R, Rhs> Expression<Rhs> for Sum<L, R> {
    fn add(self, addend: Rhs) -> Sum<Self, Rhs>
    where
        Self: Sized,
    {
        Sum(self, addend)
    }
}

impl<L, R> Reduce for Sum<L, R>
where
    L: Reduce<Output = Amount>,
    R: Reduce<Output = Amount>,
{
    type Output = Amount;

    fn reduce<E: Exchanger>(&self, exchanger: &E, dest: &Unit) -> Result<Self::Output, E::Err> {
        let (lhs, rhs) = (
            self.0.reduce(exchanger, dest)?,
            self.1.reduce(exchanger, dest)?,
        );

        Ok(Amount::new(lhs.amount + rhs.amount, lhs.unit))
    }
}

impl<L, R> Product for Sum<L, R>
where
    L: Product<Output = L>,
    R: Product<Output = R>,
{
    type Output = Self;
    fn times(self, multiplier: u32) -> Self::Output
    where
        Self: Sized,
    {
        Sum(self.0.times(multiplier), self.1.times(multiplier))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn kg() -> Unit {
        Unit::new("kg")
    }

    fn g() -> Unit {
        Unit::new("g")
    }

    #[test]
    fn unit_to_string() {
        assert_eq!(g().to_string(), "g");
        assert_eq!(kg().to_string(), "kg");
    }

    #[test]
    fn amount_to_string() {
        assert_eq!(Amount::new(1, g()).to_string(), "1g");
        assert_eq!(Amount::new(5, kg()).to_string(), "5kg");
    }

    #[test]
    fn sum_to_string() {
        let one = Amount::new(1, g()).boxed();
        let five = Amount::new(5, kg()).boxed();
        let sum = one.add(five);

        assert_eq!(sum.to_string(), "1g + 5kg");
    }

    #[test]
    fn add_amount_with_same_unit() {
        let one = Amount::new(1, g()).boxed();
        let five = Amount::new(5, g()).boxed();

        let result = one.clone().add(five.clone());
        assert_eq!(result.to_string(), "1g + 5g");
    }

    #[test]
    fn amount_multiplication() {
        let five = Amount::new(5, g()).boxed();

        let result = five.times(3);

        assert_eq!(result.to_string(), "15g");
    }

    #[test]
    fn sum_add_amount() {
        let one = Amount::new(1, g()).boxed();
        let two = Amount::new(2, g()).boxed();
        let five = Amount::new(5, kg()).boxed();

        let result = one.clone().add(five.clone());
        let result = result.add(two.clone());

        assert_eq!(result.to_string(), "1g + 5kg + 2g");
    }

    #[test]
    fn add_sum2() {
        let one = Amount::new(1, g()).boxed();
        let two = Amount::new(2, g()).boxed();
        let five = Amount::new(5, kg()).boxed();

        let sum1 = one.clone().add(five.clone());
        let sum2 = one.clone().add(two.clone());
        // compiler error: need type annotation
        let result = sum1.add(sum2);

        assert_eq!(result.to_string(), "1g + 5kg + 1g + 2g");
    }

    #[test]
    fn sum_multiplication() {
        let one = Amount::new(1, g()).boxed();
        let five = Amount::new(5, kg()).boxed();

        let result = one.clone().add(five.clone());
        let result = result.times(3);

        assert_eq!(result.to_string(), "3g + 15kg");
    }

    #[test]
    fn reduce_amount_to_same_unit() {
        let one = Amount::new(1, g()).boxed();

        let result = one.reduce(&Weight, &g()).unwrap();
        assert_eq!(result, one);
    }

    #[test]
    fn reduce_amount_to_diff_unit() {
        let one = Amount::new(1, kg());

        let result = one.reduce(&Weight, &g()).unwrap();
        assert_eq!(result, Amount::new(1000, g()));
    }

    #[test]
    fn reduce_sum_to_same_unit() {
        let one = Amount::new(1, g()).boxed();
        let five = Amount::new(5, g()).boxed();

        let sum = one.add(five);

        let result = sum.reduce(&Weight, &g()).unwrap();
        assert_eq!(result, Amount::new(6, g()));
    }

    #[test]
    fn reduce_sum_to_diff_unit() {
        let one = Amount::new(1, kg()).boxed();
        let five = Amount::new(5, g()).boxed();

        let sum = one.add(five);

        let result = sum.reduce(&Weight, &g()).unwrap();
        assert_eq!(result, Amount::new(1005, g()));
    }

    struct Weight;

    impl Exchanger for Weight {
        type Err = ();

        fn rate(&self, source: &Unit, dest: &Unit) -> Result<u32, ()> {
            match (&*source.key, &*dest.key) {
                ("kg", "g") => Ok(1000),
                _ => todo!(),
            }
        }
    }
}
