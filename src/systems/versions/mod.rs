//! Constraint Satisfaction Problems for version solving.
//!
//! @todo honestly quite a lot of this could be (and probably should be)\
//! replaced with pubgrub's version constraints [package](https://crates.io/crates/version-ranges),\
//! which seems pretty damn good.
mod allowed;
pub mod assignment;
mod bounds;
pub mod constraint;
pub mod fetching;

pub mod prelude {
  //! Common exports for package version systems
  pub use super::assignment::VersionLock;
}
