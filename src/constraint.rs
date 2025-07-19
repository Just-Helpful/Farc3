//! Traits for constraining values that variables can take

use super::assignment::Assignment;
use std::fmt::Debug;

/// A constraint that affects given variables in a system\
/// A given constriant can be satisfied by multiple possible assignments.
///
/// ## Invariants
///
/// 1. if `self.variables().count() == 0`, then `self.size() == 1`\
///    i.e. if a constraint affects no variables, it should have 1 solution, the empty solution
/// 2. if `this.variables().count() > 0`, then `this.decompositions().count() >= 1`\
///    i.e. if a constraint affects any variables, it should have at least 1 decomposition
///
/// ## Formal definition
///
/// Formally, a constraint is a set of possible assignments for the variables within a problem.
///
/// For example, in a minesweeper game:
/// - the variables would be `[x, y]` positions for a tile
/// - the values to assign would be `true`/`false` for whether a mine is in a tile
///
/// And a possible constraint could be:
/// ```json
/// {
///   { [2, 3]: true, [3, 3]: false },
///   // position [2, 3] is a mine
///   // position [3, 3] is safe
///
///   // or
///
///   { [2, 3]: false, [3, 3]: true },
///   // position [2, 3] is safe
///   // position [3, 3] is a mine
/// }
/// ```
///
/// ## Notes
///
/// I don't really like this interface for `Constraint`s,\
/// it feels like there's a fair bit of duplicated / unnecessary methods on it.\
///
/// More specifically:
/// - it feels like `Var` should be a type argument
/// - `size` feels like it's hard to explain what it does,\
///   which feels like a poorly defined interface method
/// - `decompositions` feels like it overlaps `pop_solution`\
///   and also could be better in its argument and return types
/// - `reduce` feels like its return type could provide more information
///   - it feels wrong to return a `Result<bool, _>`\
///     as we could return something more informative than `bool`
///
/// I'm going to be attempting to rework it by writing a whole bunch of examples\
/// and seeing what parts are common to most CSPs.
pub trait Constraint {
  /// The type of variables assigned by the Constraint
  type Var;
  /// Any solution to a Constraint
  type Solution: Assignment;
  /// The error raised when 2 constraints conflict with each other
  type ConflictErr: Debug;

  /// An approximation to the number of unique assignments that `self` has.
  ///
  /// This doesn't *have* to be exact, but should at least:
  /// - `return 1` when there's only one unique solution
  /// - `return 0` when no solutions are possible
  fn size(&self) -> usize;

  /// All variables that `self` affects
  fn variables(&self) -> impl Iterator<Item = Self::Var>;

  /// Possible decompositions for this constraint.
  ///
  /// A decomposition is a constraint that:
  /// 1. affects a subset of `self.variables()`
  /// 2. has a single unique solution that `self` allows
  ///
  /// ## Returns
  ///
  /// An iterator over the possible decompositions of `self`
  fn decompositions(&self) -> impl Iterator<Item = Self>;

  /// Removes the overlap between another constraint and this one.\
  /// This is useful as it lets us remove uncertainty from the system\
  /// and make it simpler to solve.
  ///
  /// ## Arguments
  ///
  /// - `other`: the other constraint to attempt reduction with
  ///
  /// ## Returns
  ///
  /// Whether `other` successfully reduced `self`,\
  /// or a conflict error if `other` conflicts with this constraint.
  ///
  /// Constraints conflict if there's **no** assignment that satisfies both.
  fn reduce(&mut self, other: &Self) -> Result<bool, Self::ConflictErr>;

  /// Pops all variables that have a unique assignment in this constraint
  fn pop_solution(&mut self) -> Option<Self::Solution>;
}
