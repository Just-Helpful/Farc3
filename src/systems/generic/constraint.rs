//! Generic constraints for variables with discrete values

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;

use crate::prelude::Constraint;
use crate::systems::generic::utils::IteratorPartition;
use crate::utils::NewHashSet;

use super::assignment::DiscreteAssignment;

/// A generic form of Constraints on discrete variables.
///
/// ## Note
///
/// This **is not** the most efficient implementation,\
/// as it effectively tries to store all possible assignments.
///
/// Whilst this lets it be as flexible as possible, it:
/// 1. uses **a lot** more space
/// 2. has much worse time complexity
///
/// Than specialised implementations of constraints.\
/// If you want a more performant implementation, check out others in [`crate::systems`].
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct DiscreteConstraint<V, T: Hash + Eq> {
  variables: Vec<V>,
  assignments: NewHashSet<Vec<T>>,
}

impl<V, T: Hash + Eq> Default for DiscreteConstraint<V, T> {
  fn default() -> Self {
    Self {
      variables: Default::default(),
      assignments: Default::default(),
    }
  }
}

impl<V: Hash + Eq + Debug, T: Hash + Eq, I: IntoIterator<Item = (V, T)>> FromIterator<I>
  for DiscreteConstraint<V, T>
{
  fn from_iter<U: IntoIterator<Item = I>>(iter: U) -> Self {
    let mut iters = iter.into_iter();
    let Some(assigns) = iters.next() else {
      return Self::default();
    };
    let (variables, values): (Vec<V>, Vec<T>) = assigns.into_iter().unzip();
    let mut assignments = NewHashSet::from([values]);

    for assigns in iters {
      let (vars0, values): (HashSet<V>, Vec<T>) = assigns.into_iter().unzip();
      assert!(
        (variables.len() == vars0.len()) && variables.iter().all(|var| vars0.contains(var)),
        "variables are not consistent when constructing generic constraint\n\
        help: expected all assignments to use the variables {variables:?}",
      );
      assignments.insert(values);
    }

    Self {
      variables,
      assignments,
    }
  }
}

impl<V: Hash + Eq + Clone, T: Hash + Eq + Clone> Constraint for DiscreteConstraint<V, T> {
  type Var = V;
  type Solution = DiscreteAssignment<V, T>;
  // @todo give a more informative error type
  type ConflictErr = ();

  fn size(&self) -> usize {
    self.assignments.len()
  }

  fn variables(&self) -> impl Iterator<Item = Self::Var> {
    self.variables.iter().cloned()
  }

  fn decompositions(&self) -> impl Iterator<Item = Self> {
    self.assignments.iter().map(|vals| Self {
      variables: self.variables.clone(),
      assignments: NewHashSet::from([vals.clone()]),
    })
  }

  fn reduce(&mut self, other: &Self) -> Result<bool, Self::ConflictErr> {
    // create a map from variables to indexes
    let vars: HashMap<&V, usize> = other
      .variables
      .iter()
      .enumerate()
      .map(|(idx, var)| (var, idx))
      .collect();

    // calculate indexes for shared variables
    let idxs: Vec<_> = self
      .variables
      .iter()
      .enumerate()
      .filter_map(|(idx0, var)| {
        let idx1 = vars.get(var)?;
        Some((idx0, *idx1))
      })
      .collect();

    let len = self.assignments.len();

    self.assignments.retain(|values0| {
      // if there's some "supporting" assignment in `other`
      // that doesn't contradict with this assignment, retain it.
      other.assignments.iter().any(|values1| {
        // whether `values1` "supports" `values0`
        idxs
          .iter()
          .map(|&(idx0, idx1)| (&values0[idx0], &values1[idx1]))
          .all(|(value0, value1)| value0 == value1)
      })
    });

    let new_len = self.assignments.len();
    if 0 < new_len {
      Ok(new_len < len)
    } else {
      Err(())
    }
  }

  fn pop_solution(&mut self) -> Option<Self::Solution> {
    let idxs = self.common_idxs()?;
    let cons = self.pop_idxs(idxs)?;
    debug_assert!(
      cons.size() == 1,
      "popped constraint should more only 1 unique solution"
    );

    let values = cons.assignments.into_iter().next()?;
    Some(cons.variables.into_iter().zip(values).collect())
  }
}

impl<V, T: Hash + Eq> DiscreteConstraint<V, T> {
  /// Finds the indexes that, for all value assignments, have the same value.
  fn common_idxs(&self) -> Option<Vec<usize>> {
    // calculate the indexes of values that are common to all assignments
    let mut optn_solution: Option<Vec<(usize, &T)>> = None;

    for values in &self.assignments {
      let Some(solution) = &mut optn_solution else {
        optn_solution = Some(values.iter().enumerate().collect());
        continue;
      };

      solution.retain(|(idx, value)| &&values[*idx] == value);
    }

    Some(optn_solution?.into_iter().map(|(idx, _)| idx).collect())
  }

  /// Pops all the variables and values at the given indexes in a constraint,\
  /// returning the new constraint created by these variables and values.
  ///
  /// ## Note
  ///
  /// This assumes that the indexes are in ascending/increasing order
  fn pop_idxs(&mut self, idxs: Vec<usize>) -> Option<DiscreteConstraint<V, T>> {
    // if there aren't any variables in this constraint
    // then we can't really pop **any** indexes from it at all.
    if self.variables.is_empty() {
      return None;
    }

    let (variables, rest) = mem::take(&mut self.variables)
      .into_iter()
      .partition_idxs(idxs.clone());
    self.variables = rest;

    let mut assignments = NewHashSet::default();
    for values in mem::take(&mut self.assignments) {
      let (values, rest) = values.into_iter().partition_idxs(idxs.clone());
      self.assignments.insert(rest);
      assignments.insert(values);
    }

    Some(DiscreteConstraint {
      variables,
      assignments,
    })
  }
}
