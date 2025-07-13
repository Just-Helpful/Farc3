//! Version range caching for packages
use super::super::bounds::LowerBound;
use super::super::bounds::{UpperBound, lower_lt_upper};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ops::Bound;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::RwLock;

/// A cache entry that stores possible versions for a package.
///
/// Invariant: no bounds overlap
pub struct VersionCacheEntry<V> {
  /// The versions available in this cache entry
  versions: BTreeSet<V>,
  /// Keeps track of the ends of all version ranges stored in this entry
  range_ends: BTreeMap<UpperBound<V>, usize>,
  /// The ranges of versions stored in this entry.
  ///
  /// This is implemented as a map from the start of the bound
  /// to the end bound and the previous bound start
  ranges: Vec<(LowerBound<V>, UpperBound<V>)>,
}

impl<V> VersionCacheEntry<V> {
  /// Finds the indexes of all ranges in `ranges` that overlap `(start, end)`
  fn indexes_to_fetch(&self, range: (Bound<V>, Bound<V>)) -> impl Iterator<Item = usize>
  where
    V: Ord,
  {
    let start = UpperBound::from(range.0);
    let end = UpperBound::from(range.1);

    // find the ranges that have ends occuring inside `(start, end)`
    let ends_within = self
      .range_ends
      .range((Bound::Included(&start), Bound::Included(&end)))
      .map(|(_, &idx)| idx);

    // add on the next bound after these if it has its start in `(start, end)`
    ends_within.chain(
      self
        .range_ends
        .range((Bound::Excluded(&end), Bound::Unbounded))
        .next()
        .map(|(_, &idx)| idx)
        .filter(|&idx| lower_lt_upper(&self.ranges[idx].0, &end)),
    )
  }

  /// Finds all ranges that need to be fetched,\
  /// in order to get all versions from `start` to `end` into cache.
  ///
  /// This avoids refetching items we've already fetched.
  ///
  /// @todo this **might** be slower than a straightforward linear search,
  /// need to benchmark both implementations and pick the faster one.
  fn to_fetch(&self, range: (Bound<V>, Bound<V>)) -> impl IntoIterator<Item = (Bound<V>, Bound<V>)>
  where
    V: Ord + Clone,
  {
    self.indexes_to_fetch(range).map(|idx| {
      let (start, end) = &self.ranges[idx];
      (start.deref().clone(), end.deref().clone())
    })
  }

  /// Inserts versions fetched for the given range into this cache entry
  fn insert(&mut self, range: (Bound<V>, Bound<V>), versions: impl IntoIterator<Item = V>) {
    // replace the ranges first
  }
}

/// A version fetcher that caches the available versions of fetched packages.
/// @todo provide a feature to replace the use of this with a dashmap
pub struct CachedVersionFetcher<V, F> {
  pub(crate) cache: Arc<RwLock<HashMap<Box<str>, VersionCacheEntry<V>>>>,
  pub(crate) fetcher: F,
}
