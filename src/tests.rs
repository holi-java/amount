use super::*;
use crate::test::*;
use std::ops::Add;

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
    let one = Amount::new(1, g());
    let five = Amount::new(5, kg());
    let sum = one.add(five);

    assert_eq!(sum.to_string(), "1g + 5kg");
}

#[test]
fn add_amount_with_same_unit() {
    let one = Amount::new(1, g());
    let five = Amount::new(5, g());

    let result = one.clone().add(five.clone());
    assert_eq!(result.to_string(), "1g + 5g");
}

#[test]
fn amount_add() {
    let g = Amount::new(1, g());
    let _ = g.clone() + g.clone();
    let _ = g.clone() + &g;
    let _ = &g + g.clone();
    let sum = &g + &g;
    let sum = g.clone() + sum;
    let sum = &g + sum;
    let sum = g.clone() + &sum;
    let _ = &g + &sum;
}

#[test]
fn amount_multiplication() {
    let five = Amount::new(5, g());

    let result = five * 3;

    assert_eq!(result.to_string(), "15g");
}

#[test]
fn sum_add_amount() {
    let one = Amount::new(1, g());
    let two = Amount::new(2, g());
    let five = Amount::new(5, kg());

    let result = one.clone().add(five.clone());
    let result = result.add(two.clone());

    assert_eq!(result.to_string(), "1g + 5kg + 2g");
}

#[test]
fn add_sum2() {
    let one = Amount::new(1, g());
    let two = Amount::new(2, g());
    let five = Amount::new(5, kg());

    let sum1 = one.clone().add(five.clone());
    let sum2 = one.clone().add(two.clone());
    let result = sum1 + sum2;

    assert_eq!(result.to_string(), "1g + 5kg + 1g + 2g");
}

#[test]
fn sum_multiplication() {
    let one = Amount::new(1, g());
    let five = Amount::new(5, kg());

    let result = one.clone().add(five.clone());
    let result = result * 3;

    assert_eq!(result.to_string(), "3g + 15kg");
}

#[test]
fn reduce_amount_to_same_unit() {
    let one = Amount::new(1, g());

    let result = Weight.reduce(&one).unwrap();
    assert_eq!(result, one);
}

#[test]
fn reduce_amount_to_diff_unit() {
    let one = Amount::new(1, kg());

    let result = Weight.reduce(one).unwrap();
    assert_eq!(result, Amount::new(1000, g()));
}

#[test]
fn reduce_sum_to_same_unit() {
    let one = Amount::new(1, g());
    let five = Amount::new(5, g());

    let sum = one.add(five);

    let result = Weight.reduce(sum).unwrap();
    assert_eq!(result, Amount::new(6, g()));
}

#[test]
fn reduce_sum_to_diff_unit() {
    let one = Amount::new(1, kg());
    let five = Amount::new(5, g());

    let sum = one.add(five);

    let result = Weight.reduce(sum).unwrap();
    assert_eq!(result, Amount::new(1005, g()));
}

#[test]
fn parse_amount_from_string() {
    assert_eq!(Amount::new(1, g()), "1g".parse().unwrap());
    assert_eq!(Amount::new(2, g()), "2g".parse().unwrap());
    assert_eq!(Amount::new(12, g()), "12g".parse().unwrap());
}
