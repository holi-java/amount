#![allow(unused)]
use amount::*;
#[path = "../src/test.rs"]
mod test;
use test::*;

#[test]
fn dyn_exchanger() {
    let exchanger =
        Box::new(Weight) as Box<dyn ExchangerExt<Rate = <Weight as Exchanger>::Rate, Err = Error>>;

    let amount = "50kg".parse::<Amount>().unwrap();
    assert_eq!(exchanger.reduce(amount).unwrap(), Amount::new(50_000, g()));
}