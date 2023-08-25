use amount::*;

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

    assert_eq!(result.reduce(&Weight), Ok(Amount::new(15006, g)));

    struct Weight;
    impl Exchanger for Weight {
        type Rate = u32;
        type Err = ();

        fn rate(&self, source: &Unit, dest: &Unit) -> Self::Output {
            match (&*source.key, &*dest.key) {
                ("kg", "g") => Ok(1000),
                _ => Err(()),
            }
        }

        fn base_unit(&self) -> Unit {
            Unit::new("g")
        }
    }
}
