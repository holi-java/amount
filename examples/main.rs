use amount::*;
use std::error::Error as StdError;
#[path = "../src/test.rs"]
mod test;
use test::CustomWeight;

fn main() -> Result<(), Box<dyn StdError>> {
    run_example_with_base_unit("g")?;
    run_example_with_base_unit("jin")?;
    run_example_with_base_unit("kg")?;
    return Ok(());

    fn run_example_with_base_unit(base: &str) -> Result<(), Box<dyn StdError>> {
        let ext = Box::new(Weight::base(base)?.extend(CustomWeight::default())?)
            as Box<dyn Exchanger<Rate = u64, Err = Error>>;

        println!("Unit Rates: {base}");
        println!("====================");
        ext.sorted_units().iter().for_each(|(unit, _)| {
            let current = Amount::new(1, unit.clone());
            let base = ext.reduce(&current).unwrap();
            println!("{current} => {base}");
        });

        let amount = format!("12345678{base}").parse::<Amount>()?;
        println!();
        println!("Exchange");
        println!("====================");
        println!("origin amount = {amount}");
        println!("base amount = {}", ext.reduce(&amount)?);
        println!("human readable amount = {}", ext.split(&amount)?);
        println!();
        println!();
        Ok(())
    }
}
