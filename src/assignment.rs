//! Traits for defining assignments to variables

/// A unique assignment of values to variables in a system
///
/// ## *Formal definition*
///
/// Formally, an assignment is a map from a variable in a problem to a value.
///
/// For example, in a minesweeper game:
/// - the variables would be `[x, y]` positions for a tile
/// - the values to assign would be `true`/`false` for whether a mine is in a tile
///
/// And a possible assignment could be:
/// ```json
/// { [2, 3]: true, [3, 3]: false }
/// // position [2, 3] is a mine
/// // position [3, 3] is safe
/// ```
pub trait Assignment {
  /// Calcultes the intersection of 2 solutions,\
  /// that only assigns a variable when:
  /// 1. the variable is assigned in both solutions
  /// 2. the values for the variable do not contradict
  fn intersection(self, other: Self) -> Self;

  /// Calulates the union of 2 solutions,\
  /// that assigns a variable when:
  /// 1. the variable is assigned in both solutions
  /// 2. the values for the variable do not contradict
  ///
  /// ## Note
  ///
  /// Whilst this implementation does **slightly** hide contradiction errors,\
  /// these are meant to be caught during constraint reduction instead.
  fn union(self, other: Self) -> Self;
}
