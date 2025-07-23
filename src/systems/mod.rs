//! Example constraints and assignments for constraint satisfaction problems
pub mod discrete;
pub mod mines;

pub mod prelude {
  //! Common exports for constraint definitions
  pub use super::{discrete::prelude::*, mines::prelude::*};
}
