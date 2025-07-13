use std::{
  cmp::Ordering,
  ops::{Bound, Deref, DerefMut},
};

/// The upper end of a range.\
/// Whilst `Ord` isn't well defined for a general `Bound`,\
/// it can be well defined if we know what end of the range the `Bound` is.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LowerBound<T>(Bound<T>);

impl<T> From<Bound<T>> for LowerBound<T> {
  fn from(value: Bound<T>) -> Self {
    LowerBound(value)
  }
}

impl<T: PartialOrd> PartialOrd for LowerBound<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (&self.0, &other.0) {
      (Bound::Excluded(s), Bound::Excluded(o)) => s.partial_cmp(o),
      (Bound::Excluded(_), _) => Some(Ordering::Greater),

      (Bound::Included(_), Bound::Unbounded) => Some(Ordering::Greater),
      (Bound::Included(s), Bound::Included(o)) => s.partial_cmp(o),
      (Bound::Included(_), Bound::Excluded(_)) => Some(Ordering::Less),

      (Bound::Unbounded, Bound::Unbounded) => Some(Ordering::Equal),
      (Bound::Unbounded, _) => Some(Ordering::Less),
    }
  }
}
impl<T: Ord> Ord for LowerBound<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap()
  }
}

impl<T> Deref for LowerBound<T> {
  type Target = Bound<T>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl<T> DerefMut for LowerBound<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> From<LowerBound<T>> for Bound<T> {
  fn from(value: LowerBound<T>) -> Self {
    value.0
  }
}

/// The upper end of a range.\
/// Whilst `Ord` isn't well defined for a general `Bound`,\
/// it can be well defined if we know what end of the range the `Bound` is.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct UpperBound<T>(Bound<T>);

impl<T> From<Bound<T>> for UpperBound<T> {
  fn from(value: Bound<T>) -> Self {
    UpperBound(value)
  }
}

impl<T: PartialOrd> PartialOrd for UpperBound<T> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    match (&self.0, &other.0) {
      (Bound::Excluded(s), Bound::Excluded(o)) => s.partial_cmp(o),
      (Bound::Excluded(_), _) => Some(Ordering::Less),

      (Bound::Included(_), Bound::Unbounded) => Some(Ordering::Less),
      (Bound::Included(s), Bound::Included(o)) => s.partial_cmp(o),
      (Bound::Included(_), Bound::Excluded(_)) => Some(Ordering::Greater),

      (Bound::Unbounded, Bound::Unbounded) => Some(Ordering::Equal),
      (Bound::Unbounded, _) => Some(Ordering::Greater),
    }
  }
}
impl<T: Ord> Ord for UpperBound<T> {
  fn cmp(&self, other: &Self) -> Ordering {
    self.partial_cmp(other).unwrap()
  }
}

impl<T> Deref for UpperBound<T> {
  type Target = Bound<T>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl<T> DerefMut for UpperBound<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T> From<UpperBound<T>> for Bound<T> {
  fn from(value: UpperBound<T>) -> Self {
    value.0
  }
}

impl<T: PartialOrd> PartialOrd<UpperBound<T>> for LowerBound<T> {
  fn partial_cmp(&self, other: &UpperBound<T>) -> Option<Ordering> {
    match (&self.0, &other.0) {
      // if one side is unbounded, it's always less
      (Bound::Unbounded, _) => Some(Ordering::Less),
      (_, Bound::Unbounded) => Some(Ordering::Less),

      // easy case, both are included, can just delegate
      (Bound::Included(s), Bound::Included(o)) => s.partial_cmp(o),

      (Bound::Included(s), Bound::Excluded(o)) => match s.partial_cmp(o)? {
        Ordering::Less => Some(Ordering::Less),
        Ordering::Equal => None,
        Ordering::Greater => todo!(),
      },

      (Bound::Excluded(s), Bound::Included(o)) => match s.partial_cmp(o)? {
        Ordering::Less => todo!(),
        Ordering::Equal => Some(Ordering::Less),
        Ordering::Greater => todo!(),
      },

      (Bound::Excluded(s), Bound::Excluded(o)) => match s.partial_cmp(o)? {
        Ordering::Less => todo!(),
        Ordering::Equal => Some(Ordering::Less),
        Ordering::Greater => todo!(),
      },
    }
  }
}

/// Whether a lower bound is less than an upper bound\
/// Honestly the logic behind this is just **nasty** to work out\
/// (read: requires a pen and paper), I might've messed up somewhere here...
///
/// I've effectively erred on the side of caution and only
pub fn lower_lt_upper<V: Ord>(lower: &Bound<V>, upper: &Bound<V>) -> bool {
  match (lower, upper) {
    (Bound::Unbounded, _) => true,
    (_, Bound::Unbounded) => true,
    (Bound::Included(l), Bound::Included(u)) => l < u,
    (Bound::Included(l), Bound::Excluded(u)) => l <= u,
    (Bound::Excluded(l), Bound::Included(u)) => l <= u,
    (Bound::Excluded(l), Bound::Excluded(u)) => l <= u,
  }
}
