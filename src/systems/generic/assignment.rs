//! Generic assignments from variables to discrete values

use std::{collections::hash_map, collections::HashMap, hash::Hash};

use crate::prelude::Assignment;

/// A generic form of discrete assigments to variables.
///
/// ## Examples
///
/// ```
/// # use farc3_csp::prelude::Assignment;
/// # use farc3_csp::systems::generic::assignment::DiscreteAssignment;
/// let assign0 = DiscreteAssignment::from([
///   ("a", true), ("b", false), ("c", true)
/// ]);
/// let assign1 = DiscreteAssignment::from([
///   ("a", true), ("b", true), ("d", true)
/// ]);
///
/// let assign2 = assign0.clone().intersection(assign1.clone());
/// assert_eq!(assign2, DiscreteAssignment::from([
///   ("a", true),
///   /* "b" has a conflict */
///   /* "c" and "d" are not the same variable */
/// ]));
///
/// let assign3 = assign0.union(assign1);
/// assert_eq!(assign3, DiscreteAssignment::from([
///   ("a", true),
///   /* "b" has a conflict */
///   ("c", true),
///   ("d", true),
/// ]));
/// ```
///
/// ## Note
///
/// This *isn't* necessarily the most efficient implementation.\
/// If you want a more efficient, try using a specialised implementation\
/// from another module in [`crate::systems`]
#[derive(Clone, Debug, Default, Eq)]
pub struct DiscreteAssignment<V: Hash + Eq, T>(HashMap<V, T>);

impl<V: Hash + Eq, T: PartialEq> PartialEq for DiscreteAssignment<V, T> {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl<V: Hash + Eq, T> FromIterator<(V, T)> for DiscreteAssignment<V, T> {
  fn from_iter<I: IntoIterator<Item = (V, T)>>(iter: I) -> Self {
    DiscreteAssignment(iter.into_iter().collect())
  }
}
impl<V: Hash + Eq, T, const N: usize> From<[(V, T); N]> for DiscreteAssignment<V, T> {
  fn from(value: [(V, T); N]) -> Self {
    Self::from_iter(value)
  }
}

impl<V: Hash + Eq, T: PartialEq> Assignment for DiscreteAssignment<V, T> {
  fn intersection(mut self, other: Self) -> Self {
    self.0.retain(|var, value| {
      let Some(value1) = other.0.get(var) else {
        return false;
      };
      &*value == value1
    });
    self
  }

  fn union(mut self, other: Self) -> Self {
    for (var, value) in other.0 {
      let Some(value1) = self.0.get(&var) else {
        self.0.insert(var, value);
        continue;
      };
      if &value != value1 {
        // conflict, remove `var` from `self`
        self.0.remove(&var);
      }
    }
    self
  }
}

impl<V: Hash + Eq, T> IntoIterator for DiscreteAssignment<V, T> {
  type Item = (V, T);
  type IntoIter = hash_map::IntoIter<V, T>;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
