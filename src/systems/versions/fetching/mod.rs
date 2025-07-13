use std::ops::Bound;

/// A value that can be used to fetch all available versions for a package.\
/// This is intended to allow for local caching of available package versions.
pub trait VersionFetcher<V> {
  /// An iterator over possible package versions
  type Versions: IntoIterator<Item = V>;

  /// Fetches all available versions for a package and version range.
  fn fetch(&self, name: &str, range: (Bound<V>, Bound<V>)) -> Self::Versions;

  /// Batches fetching of available package versions.\
  /// This is intended to allow for the use of endpoints that allow batched fetching
  fn fetch_batch<'a>(
    &self,
    batch: impl IntoIterator<Item = (&'a str, (Bound<V>, Bound<V>))>,
  ) -> impl IntoIterator<Item = Self::Versions> {
    batch
      .into_iter()
      .map(|(name, range)| self.fetch(name, range))
  }
}

impl<V, Vs: IntoIterator<Item = V>, F: Fn(&str, (Bound<V>, Bound<V>)) -> Vs> VersionFetcher<V>
  for F
{
  type Versions = Vs;
  fn fetch(&self, name: &str, range: (Bound<V>, Bound<V>)) -> Vs {
    self(name, range)
  }
}

mod caching;
