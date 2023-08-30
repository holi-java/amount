use amount::{custom, Amount, Error, Exchanger, ExchangerExt, Unit};
fn main() -> Result<(), Error> {
    let weight = custom(
        "jin",
        [
            (Unit::new("car"), 500_000_u64),
            (Unit::new("box"), 5_000_u64),
            (Unit::new("bag"), 80_u64),
        ],
    )?;

    println!("Potato Units");
    println!("====================");
    weight
        .sorted_units()
        .iter()
        .for_each(|(unit, rate)| println!("{unit} => {rate}"));

    let amount = Amount::new(12345678, Unit::new("kg"));

    println!();
    println!("Exchange");
    println!("====================");
    println!("origin amount = {amount}", amount = amount);
    println!("base amount = {amount}", amount = weight.reduce(&amount)?);
    println!(
        "human readable amount = {amount}",
        amount = weight.split(&amount)?
    );
    println!();
    println!();

    Ok(())
}
