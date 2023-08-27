use amount::*;
use std::error::Error as StdError;
#[path = "../src/test.rs"]
mod test;
use test::CustomWeight;

fn main() -> Result<(), Box<dyn StdError>> {
    let ext =
        Box::new(Weight.extend(CustomWeight)) as Box<dyn ExchangerExt<Rate = u64, Err = Error>>;
    println!("Unit Rates");
    println!("====================");
    ext.units().iter().for_each(|(unit, _)| {
        let current = Amount::new(1, unit.clone());
        let base = ext.reduce(&current).unwrap();
        println!("{current} => {base}");
    });

    let amount = "12345678kg".parse::<Amount>()?;
    println!();
    println!("Exchange");
    println!("====================");
    println!("amount = {amount}");
    println!("base amount = {}", ext.reduce(&amount)?);
    println!("human readable amount = {}", ext.split(&amount)?);
    Ok(())
}
