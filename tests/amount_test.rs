use amount::*;
#[path = "../src/test.rs"]
mod test;
use test::Weight;

#[test]
fn parse_amount_from_string() {
    let amount = "5kg".parse::<Amount>().unwrap();

    assert_eq!(amount, Amount::new(5, Unit::new("kg")));
}

#[test]
fn arithmetic() {
    let five = "5kg".parse::<Amount>().unwrap();
    let two = "2g".parse::<Amount>().unwrap();

    let sum = five + two;
    let result = sum * 3;

    let g = Unit::new("g");

    assert_eq!(Weight.reduce(result), Ok(Amount::new(15006, g)));
}
