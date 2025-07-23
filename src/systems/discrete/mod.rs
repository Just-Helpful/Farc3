//! Generic Constraints for CSPs with discrete variables.\
//! These **do not** have efficient implementations.
pub mod assignment;
pub mod constraint;
pub mod utils;

pub mod prelude {
  //! Common exports for generic constraint systems
  pub use super::constraint::DiscreteConstraint;
}

#[cfg(test)]
mod test;
