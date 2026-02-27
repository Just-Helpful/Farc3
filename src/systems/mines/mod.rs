//! Constraint Satisfaction Problems for minesweeper games.
pub mod assignment;
pub mod constraint;
pub mod errors;
pub mod utils;

pub mod prelude {
  //! Common exports for minesweeper systems
  pub use super::assignment::MineAssignment;
  pub use super::constraint::MineConstraint;
}

#[cfg(test)]
mod test;
