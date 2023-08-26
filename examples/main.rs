use amount::*;
use std::error::Error as StdError;
#[path = "../src/test.rs"]
mod test;
use test::CustomWeight;

fn main() -> Result<(), Box<dyn StdError>> {
    let ext = Weight.extend(CustomWeight);
    println!("Unit Exchange");
    println!("====================");
    ext.sorted_units().iter().for_each(|unit| {
        let current = Amount::new(1, unit.clone());
        let base = ext.reduce(&current).unwrap();
        println!("{current} => {base}");
    });

    let amount = "12345678kg".parse::<Amount>()?;
    println!();
    println!("Exchange");
    println!("====================");
    println!("amount = {amount}");
    println!("base = {}", ext.reduce(&amount)?);
    println!("human = {}", ext.split(&amount)?);
    Ok(())
}
