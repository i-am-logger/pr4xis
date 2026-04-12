pub mod consistency;
pub mod durability;
pub mod ontology;
pub mod volatility;

pub use ontology::*;

#[cfg(test)]
mod tests;
