use amount::*;
use std::{env::args, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let amount = args().nth(1).ok_or("empty")?.parse::<Amount>()?;
    let exchanger = &Weight;
    println!("{}", exchanger.reduce(amount.clone())?);
    println!("{}", exchanger.split(amount.clone())?);
    Ok(())
}
