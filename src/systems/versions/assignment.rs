//! Assignments for package versions
use std::collections::HashMap;

use crate::prelude::Assignment;

/// The locked versions of packages, with a single package version per package.
pub struct VersionLock<V> {
  versions: HashMap<Box<str>, V>,
}

impl<V: PartialEq> Assignment for VersionLock<V> {
  fn intersection(mut self, other: Self) -> Self {
    self.versions.retain(|name, version| {
      let Some(version0) = other.versions.get(name) else {
        return false;
      };

      version == version0
    });
    self
  }

  fn union(mut self, other: Self) -> Self {
    for (name, version) in other.versions {
      let Some(version0) = self.versions.get(&name) else {
        self.versions.insert(name, version);
        continue;
      };

      if &version != version0 {
        self.versions.remove(&name);
      }
    }
    self
  }
}
