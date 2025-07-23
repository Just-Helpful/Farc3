//! Utilities for constraint systems with discrete values

/// Additional utility methods for partitioning iterators
pub trait IteratorPartition: Iterator + Sized {
  /// Picks given indexes from an iterator.
  ///
  /// ## Arguments
  ///
  /// - `idxs`: the indexes to pick, must be in ascending order
  fn partition_idxs<B>(self, idxs: impl IntoIterator<Item = usize>) -> (B, B)
  where
    B: Default + Extend<Self::Item>,
  {
    let mut idx = 0;
    let mut iter = idxs.into_iter();
    let mut optn_ptr = iter.next();

    self.partition(|_| match optn_ptr {
      None => {
        idx += 1;
        false
      }
      Some(ptr) if idx < ptr => {
        idx += 1;
        false
      }
      _ => {
        // advance `ptr`
        optn_ptr = iter.next();
        idx += 1;
        true
      }
    })
  }
}

impl<I: Iterator + Sized> IteratorPartition for I {}
