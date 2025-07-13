use crate::systems::versions::bounds::lower_lt_upper;

use super::bounds::{LowerBound, UpperBound};
use std::{
  collections::BTreeMap,
  ops::{Bound, Range},
};

/// A constraint for the allowed versions of a given package
///
/// **Invariants:**
/// - all ranges in this constraint do not overlap.
/// - all ranges are stored in order
#[derive(Default, Clone)]
pub struct AllowedVersions<V> {
  /// The ranges of versions stored in this entry.
  ///
  /// This is implemented as a map from the start of the bound
  /// to the end bound and the previous bound start
  ranges: Vec<(LowerBound<V>, UpperBound<V>)>,
}

impl<V> AllowedVersions<V> {
  /// Finds the index range of all version ranges that overlap `(start, end)`
  /// More precisely, we want all ranges that:
  /// 1. have an end bound occuring after `start`
  /// 2. have a start bound occuring before `end`
  ///
  /// we could use a linear search to find the start and end,
  /// and this might actually be better for a small number of ranges
  /// but we'll likely end up using this for caching, so we might
  /// end up with a lot of ranges stored.
  fn overlapping(&self, (start, end): (Bound<V>, Bound<V>)) -> Range<usize>
  where
    V: Ord,
  {
    let start_res = self.ranges.binary_search_by(|(lower, upper)| {});
  }

  fn remove_overlapping(&self) -> impl Iterator<Item = (Bound<V>, Bound<V>)> {}
}

impl<V> Extend<(Bound<V>, Bound<V>)> for AllowedVersions<V> {
  fn extend<T: IntoIterator<Item = (Bound<V>, Bound<V>)>>(&mut self, iter: T) {
    for (start, end) in iter {}
  }
}
