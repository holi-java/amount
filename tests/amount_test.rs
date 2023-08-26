use amount::*;
#[path = "../src/test.rs"]
mod test;
use test::*;

#[test]
fn parse_amount_from_string() {
    let amount = "5kg".parse::<Amount>().unwrap();

    assert_eq!(amount, Amount::new(5, Unit::new("kg")));
}

#[test]
fn reduce_to_base_unit() {
    let five = "5kg".parse::<Amount>().unwrap();
    let two = "2g".parse::<Amount>().unwrap();

    let sum = five + two;
    let result = sum * 3;

    assert_eq!(Weight.reduce(result), Ok(Amount::new(15006, g())));
}

#[test]
fn split_amount_with_base_unit() {
    let g_2200 = "2200g".parse::<Amount>().unwrap();

    let result = Weight.split(g_2200).unwrap();

    assert_eq!(result.to_string(), "2kg + 200g");
}

#[test]
fn split_amount() {
    let g_2200 = "1kg".parse::<Amount>().unwrap();

    let result = Weight.split(g_2200).unwrap();

    assert_eq!(result.to_string(), "1kg");
}

#[test]
fn reduce_split() {
    let g_2200 = "2200g".parse::<Amount>().unwrap();

    let result = Weight.split(&g_2200).unwrap();

    assert_eq!(Weight.reduce(result).unwrap(), g_2200);
}

#[test]
fn split_multiplication() {
    let g_2300 = "2300g".parse::<Amount>().unwrap();

    let result = Weight.split(g_2300).unwrap();

    let result = result * 3;
    assert_eq!(result.to_string(), "6kg + 900g");

    let _ = &result * 3;
}

#[test]
fn split_add() {
    let g_2300 = "2300g".parse::<Amount>().unwrap();

    let split = Weight.split(&g_2300).unwrap();

    let sum = &split + &split;
    assert_eq!(sum.to_string(), "2kg + 300g + 2kg + 300g");

    let sum = &split + sum;
    assert_eq!(sum.to_string(), "2kg + 300g + 2kg + 300g + 2kg + 300g");

    let _ = &split + &g_2300;
    let _ = &sum + &split;
    let _ = &g_2300 + &split;
}

#[test]
fn split_into_iter() {
    let g_2300 = "2300g".parse::<Amount>().unwrap();

    let split = Weight.split(g_2300).unwrap();

    assert_eq!(
        split.into_iter().collect::<Vec<_>>(),
        [Amount::new(2, kg()), Amount::new(300, g())]
    );
}

#[test]
fn split_as_iter() {
    let g_2300 = "2300g".parse::<Amount>().unwrap();

    let split = Weight.split(g_2300).unwrap();

    assert_eq!(
        split.iter().collect::<Vec<_>>(),
        [&Amount::new(2, kg()), &Amount::new(300, g())]
    );
}
