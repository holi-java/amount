use amount::*;
#[path = "../src/test.rs"]
mod test;
use test::*;

#[test]
fn exchanger_without_base_unit() {
    define_exchanger!(
        #[base_unit = "g"]
        #[derive(Debug, Clone)]
        Weight {
            kg = 1_000
        }
    );

    let weight = Weight::base("kg").unwrap();
    let cloned = weight.clone();
    assert_eq!(weight.units(), cloned.units());

    assert_eq!(Weight::units(), ["kg", "g"]);
    assert_eq!(weight.units(), [(kg(), 1)]);
    assert_eq!(*weight.base_unit(), kg());

    let weight = Weight::base("g").unwrap();
    assert_eq!(weight.units(), [(kg(), 1_000), (g(), 1)]);
    assert_eq!(*weight.base_unit(), g());
}
