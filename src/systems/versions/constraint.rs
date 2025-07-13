use std::{collections::HashMap, ops::Bound};

use crate::{
  prelude::{Constraint, VersionLock},
  systems::versions::VersionFetcher,
};

/// A constraint for the versions of various packages,\
/// as defined in a package manifest.
pub struct VersionConstraint<V, F> {
  constraint: HashMap<Box<str>, (Bound<V>, Bound<V>)>,
  available: HashMap<Box<str>, V>,
  fetcher: F,
}

impl<V: PartialEq, F> Constraint for VersionConstraint<V> {
  type Var = Box<str>;
  type Solution = VersionLock<V>;
  type ConflictErr = ();

  fn size(&self) -> usize {}
}
