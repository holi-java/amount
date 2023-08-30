#![doc(html_playground_url = "https://play.rust-lang.org/")]
#![cfg_attr(test, feature(assert_matches))]
#![feature(try_trait_v2)]
#![feature(associated_type_defaults)]
#![feature(decl_macro)]
//!
//!
//! # Example
//!
//! [Reduce] sum of the two amounts to [Amount] with base [unit](Unit).
//!
//! ```rust
//! # use crate::amount::*;
//! # mod test;
//! # use test::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let exchanger = Weight::default().extend(CustomWeight::default())?;
//! let five = "5kg".parse::<Amount>()?;
//! let two = "2g".parse::<Amount>()?;
//!
//! let sum = five + two;
//! let result = sum * 3;
//!
//! let result = exchanger.reduce(result)?;
//! assert_eq!(result, Amount::new(15006, exchanger.base_unit().clone()));
//!
//! let result = exchanger.split(result)?.into_iter().collect::<Vec<_>>();
//! assert_eq!(result, [Amount::new(15, kg()), Amount::new(6, g())]);
//! # Ok(())
//! # }
//! ```
#[macro_use]
mod macros;
mod amount;
mod split;
mod sum;
mod table;
#[cfg(any(test, doctest))]
pub mod test;
#[cfg(test)]
mod tests;
mod traits;
mod weight;

pub use amount::*;
pub use split::*;
pub use table::*;
pub use traits::*;
pub use weight::*;
