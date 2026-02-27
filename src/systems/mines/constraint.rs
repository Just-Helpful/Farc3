//! Constraints for mine sweeper solving

use std::hash::Hash;
use std::mem;

use super::{assignment::MineAssignment, errors::MineConflicts, utils::choose_num};
use crate::{constraint::Constraint, utils::NewHashSet};

/// A constraint for the number of mines present in the given tiles.
#[derive(Default, Debug, Hash, PartialEq, Eq, Clone)]
pub struct MineConstraint<V: Hash + Eq> {
  /// The tiles that mines could be present in
  tiles: NewHashSet<V>,
  /// The number of mines assigned by this constraint
  count: usize,
}

impl<V: Hash + Eq> MineConstraint<V> {
  /// Constructs a mine constraint
  ///
  /// ## Arguments
  ///
  /// - `tiles`: the tiles that mines could be present in
  /// - `count`: the number of mines present in `tiles`
  ///
  /// ## Returns
  ///
  /// A new [`MineConstraint`]
  ///
  /// ## Examples
  ///
  /// ```
  /// # use farc3::prelude::Constraint;
  /// # use farc3::systems::mines::constraint::MineConstraint;
  /// MineConstraint::new([0, 1, 2], 2);
  /// MineConstraint::new(vec![0, 1, 2], 2);
  ///
  /// let cons = MineConstraint::new([0, 1, 2], 2);
  /// MineConstraint::new(cons.variables(), 1);
  ///
  /// MineConstraint::new(["a", "b", "c"], 2);
  /// MineConstraint::new([true, false], 1);
  /// ```
  pub fn new(tiles: impl IntoIterator<Item = V>, count: usize) -> Self {
    Self {
      tiles: NewHashSet::from_iter(tiles),
      count,
    }
  }
}

impl<V: Hash + Eq + Clone> Constraint for MineConstraint<V> {
  type Var = V;
  type Solution = MineAssignment<V>;
  type ConflictErr = MineConflicts;

  fn size(&self) -> usize {
    choose_num(self.tiles.len(), self.count)
  }

  fn variables(&self) -> impl Iterator<Item = Self::Var> {
    self.tiles.iter().cloned()
  }

  fn decompositions(&self) -> impl Iterator<Item = Self> {
    self.tiles.iter().flat_map(|tile| {
      let tiles = NewHashSet::from([tile.clone()]);

      let mut assigns = vec![];
      if self.count > 0 {
        assigns.push(Self {
          tiles: tiles.clone(),
          count: 1,
        })
      }
      if self.count < self.tiles.len() {
        assigns.push(Self {
          tiles: tiles.clone(),
          count: 0,
        })
      }

      assigns.into_iter()
    })
  }

  fn reduce(&mut self, other: &Self) -> Result<bool, Self::ConflictErr> {
    let tiles: NewHashSet<_> = self.tiles.difference(&other.tiles).cloned().collect();

    // there's several cases in which we can reduce:
    // 1. `other` is all safe tiles
    if other.count == 0 {
      // conflict when reduction would give us more mines than tiles
      if tiles.len() < self.count {
        return Err(MineConflicts);
      }

      self.tiles = tiles;
      return Ok(true);
    }

    // 2. `other` is all mine tiles
    if other.count == other.tiles.len() {
      let len_overlap = self.tiles.intersection(&other.tiles).count();

      // conflict when reduction would give us less than 0 mines
      if self.count < len_overlap {
        return Err(MineConflicts);
      }

      self.count -= len_overlap;
      self.tiles = tiles;
      return Ok(true);
    }

    // 3. `other` is a subset of `self`
    if other.tiles.is_subset(&self.tiles) {
      // conflict on either < 0 or > len number of mines
      if (self.count < other.count) || (tiles.len() < self.count - other.count) {
        return Err(MineConflicts);
      }

      self.count -= other.count;
      self.tiles = tiles;
      return Ok(true);
    }

    Ok(false)
  }

  fn pop_solution(&mut self) -> Option<Self::Solution> {
    if self.count == 0 {
      let tiles = mem::take(&mut self.tiles);
      return Some(Self::Solution::all_safe(tiles));
    }

    if self.count == self.tiles.len() {
      let tiles = mem::take(&mut self.tiles);
      self.count = 0;
      return Some(Self::Solution::all_mine(tiles));
    }

    None
  }
}
