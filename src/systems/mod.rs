//! Example constraints and assignments for constraint satisfaction problems
pub mod generic;
pub mod mines;
pub mod versions;

pub mod prelude {
  //! Common exports for constraint definitions
  pub use super::{generic::prelude::*, mines::prelude::*, versions::prelude::*};
}
