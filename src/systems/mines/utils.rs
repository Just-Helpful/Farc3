//! Utilities for mine assignment constraints

/// Returns the number of ways to choose `r` unordered items from `n` total items
///
/// ## Arguments
///
/// - `n`: how many items are available to choose from
/// - `r`: how many items should be chosen
#[inline]
pub fn choose_num(n: usize, r: usize) -> usize {
  debug_assert!(
    r <= n,
    "Unable to choose more than {r} items from a collection with {n} items",
  );
  // n! / r!
  let pick = (((r + 1).max(2))..=n).product::<usize>();
  // (n - r)!
  let fact = (2..=(n - r)).product::<usize>();

  // n! / (r! (n - r)!)
  pick / fact
}
