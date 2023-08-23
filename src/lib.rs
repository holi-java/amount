use std::fmt::{Debug, Display};

trait Boxed {
    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}

impl<T> Boxed for T {}

type DynExpression = Box<dyn Expression>;
#[derive(Debug)]
enum Error {}

trait Exchanger {
    fn rate(&self, source: &Unit, dest: &Unit) -> Result<u32, Error>;
}

trait Expression: Debug + Display {
    fn add(self: Box<Self>, addend: DynExpression) -> DynExpression;

    fn times(self: Box<Self>, multiplier: u32) -> DynExpression;

    fn reduce(&self, exchanger: &dyn Exchanger, dest: &Unit) -> Result<Amount, Error>;
}

impl Expression for DynExpression {
    fn add(self: Box<Self>, addend: DynExpression) -> DynExpression {
        (*self).add(addend)
    }

    fn times(self: Box<Self>, multiplier: u32) -> DynExpression {
        (*self).times(multiplier)
    }

    fn reduce(&self, exchanger: &dyn Exchanger, dest: &Unit) -> Result<Amount, Error> {
        (**self).reduce(exchanger, dest)
    }
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

impl Expression for Amount {
    fn add(self: Box<Self>, addend: DynExpression) -> DynExpression {
        Sum(self as DynExpression, addend).boxed()
    }

    fn times(self: Box<Self>, multiplier: u32) -> DynExpression {
        Amount::new(self.amount * multiplier, self.unit).boxed()
    }

    fn reduce(&self, exchanger: &dyn Exchanger, dest: &Unit) -> Result<Amount, Error> {
        if self.unit == *dest {
            return Ok(self.clone());
        }
        let rate = exchanger.rate(&self.unit, dest)?;
        Ok(Amount::new(self.amount * rate, dest.clone()))
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

impl<L, R> Expression for Sum<L, R>
where
    L: Expression + 'static,
    R: Expression + 'static,
{
    fn add(self: Box<Self>, addend: DynExpression) -> DynExpression {
        Sum(self as DynExpression, addend).boxed()
    }

    fn times(self: Box<Self>, multiplier: u32) -> DynExpression {
        Sum(
            self.0.boxed().times(multiplier),
            self.1.boxed().times(multiplier),
        )
        .boxed()
    }

    fn reduce(&self, exchanger: &dyn Exchanger, dest: &Unit) -> Result<Amount, Error> {
        let (lhs, rhs) = (
            self.0.reduce(exchanger, dest)?,
            self.1.reduce(exchanger, dest)?,
        );
        Ok(Amount::new(lhs.amount + rhs.amount, dest.clone()))
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
    fn sum_multiplication() {
        let one = Amount::new(1, g()).boxed();
        let five = Amount::new(5, kg()).boxed();

        let result = one.clone().add(five.clone()).times(3);

        assert_eq!(result.to_string(), "3g + 15kg");
    }

    #[test]
    fn reduce_amount_to_same_unit() {
        let one = Amount::new(1, g()).boxed();

        let result = one.reduce(&Weight, &g()).unwrap();
        assert_eq!(result, *one);
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
        fn rate(&self, source: &Unit, dest: &Unit) -> Result<u32, Error> {
            match (&*source.key, &*dest.key) {
                ("kg", "g") => Ok(1000),
                _ => todo!(),
            }
        }
    }
}
