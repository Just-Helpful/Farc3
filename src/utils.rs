use std::{
  collections::hash_set,
  collections::HashSet,
  hash::{DefaultHasher, Hash, Hasher},
  num::Wrapping,
  ops::{Deref, DerefMut},
};

/// A Newtype wrapper on [`HashSet`] that supports `Hash`
///
/// Hashing algorithm from [stackoverflow](https://stackoverflow.com/a/77085302)
#[derive(Clone, Debug)]
pub struct NewHashSet<T>(HashSet<T>);

impl<T> Default for NewHashSet<T> {
  fn default() -> Self {
    Self(Default::default())
  }
}

impl<T: Hash + Eq> FromIterator<T> for NewHashSet<T> {
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    Self(iter.into_iter().collect())
  }
}
impl<T: Hash + Eq, const N: usize> From<[T; N]> for NewHashSet<T> {
  fn from(value: [T; N]) -> Self {
    Self::from_iter(value)
  }
}
impl<T: Hash + Eq> From<HashSet<T>> for NewHashSet<T> {
  fn from(value: HashSet<T>) -> Self {
    Self(value)
  }
}

impl<T> Deref for NewHashSet<T> {
  type Target = HashSet<T>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}
impl<T> DerefMut for NewHashSet<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<T: Hash + Eq> PartialEq for NewHashSet<T> {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}
impl<T: Hash + Eq> Eq for NewHashSet<T> {}

impl<T: Hash> Hash for NewHashSet<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    let mut sum = Wrapping::default();
    for value in &self.0 {
      let mut hasher = DefaultHasher::new();
      Hash::hash(value, &mut hasher);
      sum += hasher.finish();
    }
    state.write_u64(sum.0);
  }
}

impl<T> From<NewHashSet<T>> for HashSet<T> {
  fn from(value: NewHashSet<T>) -> Self {
    value.0
  }
}

impl<T> IntoIterator for NewHashSet<T> {
  type Item = T;
  type IntoIter = hash_set::IntoIter<T>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}
impl<'a, T> IntoIterator for &'a NewHashSet<T> {
  type Item = &'a T;
  type IntoIter = hash_set::Iter<'a, T>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}
