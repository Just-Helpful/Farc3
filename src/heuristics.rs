//! Traits for informing which constraints to explore first in a search

use super::constraint::Constraint;

/// A heuristic that guides which constraints to explore first\
/// whilst searching for solutions to systems of constraints.
///
/// The constraint with the **maximum** heuristic value is explored first
pub trait Heuristic<C> {
  /// A ranking used to decide the best constraint to explore
  type Rank: Ord;

  /// Generates a rank for the given `Constraint` in its System.
  ///
  /// ## Arguments
  ///
  /// - `constraint`: the constraint to generate a ranking for
  /// - `overlaps`: all constraints that share the same variables as `constraint`
  ///
  /// ## Returns
  ///
  /// An orderable ranking for the given constraint.\
  /// [`System`] will pick the constraint with the **max** rank
  fn rank(&mut self, constraint: &C, overlaps: &[&C]) -> Self::Rank;
}

impl<C, R: Ord, H: FnMut(&C, &[&C]) -> R> Heuristic<C> for H {
  type Rank = R;
  fn rank(&mut self, constraint: &C, overlaps: &[&C]) -> Self::Rank {
    self(constraint, overlaps)
  }
}

/// A default heuristic for ranking constraints.\
/// This prioritises constraints that:\
/// 1. have the minimum possible assignments
/// 2. affect the maximum number of other constraints
#[derive(Default)]
pub struct DefaultHeuristic;

impl<C: Constraint> Heuristic<C> for DefaultHeuristic {
  type Rank = (isize, usize);

  fn rank(&mut self, constraint: &C, overlaps: &[&C]) -> Self::Rank {
    (-(constraint.size() as isize), overlaps.len())
  }
}
