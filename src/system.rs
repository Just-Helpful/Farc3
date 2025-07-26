//! A generic constraint solving algorithm for a system of constraints

use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem::{self, MaybeUninit};
use std::{slice, vec};

use crate::{
  assignment::Assignment,
  constraint::Constraint,
  heuristics::{DefaultHeuristic, Heuristic},
};

/// A Generic constraint system.
///
/// Constraint systems should in general act like sets of constraints,\
/// but with the ability to produce solutions to those sets of constraints.
///
/// ## Invariants
///
/// There are several key invariants that will be maintained by this [`System`]:
///
/// 1. There are no duplicate constraints in [`Self::constraints`]
/// 2. For each variable that a constraint affects, there's a back reference in [`Self::references`]
///
/// ## Todo
///
/// ### Rc rewrite
///
/// We can probably refactor this to make use of `Rc`s and `Mutex`es,\
/// instead of relying on a hashmap and backlinks. This will:
/// 1. require less indirection (no hopping through 2 hashmaps)
/// 2. not require multiple hashes to access connected constraints
/// 3. maybe remove the need for `System::references`
/// 4. probably simplify a few methods :P
///
/// ### Set-like methods
///
/// It'd probably be useful to provide more of the methods\
/// commonly seen on `HashSet`s, i.e. `union` and `intersection`.\
/// These methods could be used to combine constraint systems\
/// and I see them maybe forming something that users may need.
///
/// I should focus on developing these in a separate branch\
/// as it's not a feature that is core to the implementation\
/// of constraint systems, but *could be* helpful.
///
/// ### Assignment caching
///
/// We ideally want to be able to avoid doing repeated work for solutions.\
/// This would primarily consist of caching which assignments have resulted\
/// in conflicts and valid solutions to the [`System`], which can be acheived\
/// by creating a [`HashMap`] that maps from seen assignments to their\
/// eventual solution / conflict.
///
/// This can potentially be extended by introducing "causality", where\
/// the order that the assignments were arrived at is tracked, to allow\
/// us to determine which assignments caused others.\
/// (for an example, see `pubgrub` incompatability explanations)
///
/// This can also better inform which constraint to search next,\
/// by examining the frequency with which supersets of that assignment\
/// resolve to solutions.\
/// (if supersets commonly lead to solutions, the assignment is probably "good")
#[derive(Clone, Debug)]
pub struct System<C: Constraint> {
  /// Constraints to be solved.
  ///
  /// We use a Vec here for memory size reasons\
  /// and better support for operations on slices.
  ///
  /// Admittedly, we could use a hashmap here,\
  /// which would simplify some of the methods.\
  /// I should just benchmark this properly.
  constraints: Vec<C>,
  /// The indexes of added constraints
  ///
  /// This is used make constraint removal simpler, avoid duplicates\
  /// and "hide" the fact we're using a `Vec` under the hood
  idx_map: HashMap<u64, usize>,
  /// Back-references to aid in solving this constraint system
  references: HashMap<C::Var, HashSet<usize>>,
  /// Constraints to start minimisation from
  to_minimise: BTreeSet<usize>,
}

/*------------------------------------------------
-                  Constructors                  -
------------------------------------------------*/
impl<C: Constraint> Default for System<C> {
  fn default() -> Self {
    Self {
      constraints: Default::default(),
      idx_map: Default::default(),
      references: Default::default(),
      to_minimise: Default::default(),
    }
  }
}

impl<C: Constraint> Extend<C> for System<C>
where
  C: Hash + Eq,
  C::Var: Hash + Eq,
{
  fn extend<T: IntoIterator<Item = C>>(&mut self, iter: T) {
    let constraints: Vec<C> = iter.into_iter().collect();
    let len = self.constraints.len();
    let range = len..len + constraints.len();

    self.idx_map.extend(
      constraints.iter().map(default_hash).zip(range.clone()), //
    );

    for (idx, cons) in range.clone().zip(&constraints) {
      for var in cons.variables() {
        self.references.entry(var).or_default().insert(idx);
      }
    }

    self.to_minimise.extend(range);
    self.constraints.extend(constraints);
  }
}

impl<C: Constraint> FromIterator<C> for System<C>
where
  C: Hash + Eq,
  C::Var: Hash + Eq,
{
  fn from_iter<T: IntoIterator<Item = C>>(iter: T) -> Self {
    let mut sys = Self::default();
    sys.extend(iter);
    sys
  }
}

impl<C: Constraint, const N: usize> From<[C; N]> for System<C>
where
  C: Hash + Eq,
  C::Var: Hash + Eq,
{
  fn from(value: [C; N]) -> Self {
    Self::from_iter(value)
  }
}

/// Helper method to hash a value with the default hasher
#[inline(always)]
fn default_hash<T: Hash + Eq>(value: T) -> u64 {
  let mut hasher = DefaultHasher::default();
  value.hash(&mut hasher);
  hasher.finish()
}

impl<C: Constraint> System<C> {
  /// Whether system contains no constraints (i.e. is unconstrained)
  pub fn is_empty(&self) -> bool {
    self.constraints.is_empty()
  }

  /// The number of constraints currently in the system
  pub fn len(&self) -> usize {
    self.constraints.len()
  }
}

impl<C: Constraint> IntoIterator for System<C> {
  type Item = C;
  type IntoIter = vec::IntoIter<C>;
  fn into_iter(self) -> Self::IntoIter {
    self.constraints.into_iter()
  }
}
impl<'a, C: Constraint> IntoIterator for &'a System<C> {
  type Item = &'a C;
  type IntoIter = slice::Iter<'a, C>;
  fn into_iter(self) -> Self::IntoIter {
    self.constraints.iter()
  }
}

/*------------------------------------------------
-                Set-Like methods                -
------------------------------------------------*/
impl<C: Constraint> System<C> {
  /// Adds a constraint to `self`, further restricting the possible solutions.
  ///
  /// ## Arguments
  ///
  /// - `constraint`: the constraint to be added
  ///
  /// ## Returns
  ///
  /// Whether the constraint already existed in the system
  pub fn insert(&mut self, constraint: C) -> bool
  where
    C: Hash + Eq,
    C::Var: Hash + Eq,
  {
    let hash = default_hash(&constraint);
    if self.idx_map.contains_key(&hash) {
      return true;
    }

    // update references
    let idx = self.constraints.len();
    for var in constraint.variables() {
      self.references.entry(var).or_default().insert(idx);
    }

    // log that we've seen the constraint
    self.idx_map.insert(hash, idx);

    // add constraint
    self.constraints.push(constraint);
    self.to_minimise.insert(idx);
    false
  }

  /// Removes the constraint at a given index from the system and returns it
  fn remove_idx(&mut self, idx: usize) -> Option<C>
  where
    C::Var: Hash + Eq,
  {
    // short circuit on no constraints
    if self.constraints.is_empty() {
      return None;
    }

    // remove all references to the last constraint
    let last_idx = self.constraints.len() - 1;
    for var in self.constraints[last_idx].variables() {
      let Some(idxs) = self.references.get_mut(&var) else {
        continue;
      };
      idxs.remove(&last_idx);
    }

    // if constraint happens to be at end,
    // we don't need to swap remove and can just `pop` instead
    if idx == last_idx {
      return self.constraints.pop();
    }

    // swap the positions of the constraints.
    for var in self.constraints[idx].variables() {
      let Some(idxs) = self.references.get_mut(&var) else {
        continue;
      };
      idxs.remove(&idx);
    }
    self.constraints.swap(idx, last_idx);
    for var in self.constraints[idx].variables() {
      let Some(idxs) = self.references.get_mut(&var) else {
        continue;
      };
      idxs.insert(idx);
    }

    self.constraints.pop()
  }

  /// Removes a constraint from `self`, allowing for more possible solutions
  ///
  /// ## Arguments
  ///
  /// - `constraint`: a reference to the constraint to remove
  ///
  /// ## Returns
  ///
  /// The existing contraint, popped from the system
  pub fn remove(&mut self, constraint: &C) -> Option<C>
  where
    C: Hash + Eq,
    C::Var: Hash + Eq,
  {
    let hash = default_hash(constraint);
    let &idx = self.idx_map.get(&hash)?;
    self.remove_idx(idx)
  }

  /// Queues all constraints to be minimised.\
  /// Call this if you've done something **really weird** to the `System`\
  /// and want to ensure that constraints are correctly minimised.
  pub fn queue_all(&mut self) -> &mut Self {
    self.to_minimise = (0..self.constraints.len()).collect();
    self
  }
}

/*------------------------------------------------
-             System solving methods             -
------------------------------------------------*/
impl<C: Constraint> System<C> {
  /// Pops the solution for all decided variables in `self`.
  pub fn pop_solution(&mut self) -> Result<C::Solution, C::ConflictErr>
  where
    C: Hash + Eq,
    C::Var: Hash + Eq,
    C::Solution: Default,
  {
    if !self.to_minimise.is_empty() {
      self.minimise()?;
    }

    let mut solution = C::Solution::default();
    for (idx, constraint) in self.constraints.iter_mut().enumerate() {
      let mut to_remove: HashSet<_> = constraint.variables().collect();

      // pop and add the solution
      let Some(sol) = constraint.pop_solution() else {
        continue;
      };
      solution = solution.union(sol);

      // clean up lingering references
      for var in constraint.variables() {
        to_remove.remove(&var);
      }
      for var in to_remove {
        let Some(idxs) = self.references.get_mut(&var) else {
          continue;
        };
        idxs.remove(&idx);
      }
    }

    self.remove_empty();

    Ok(solution)
  }

  /// Removes all empty constraints from the system.\
  /// This is mostly used to keep the system small and reduce time complexity.
  fn remove_empty(&mut self)
  where
    C: Hash + Eq,
    C::Var: Hash + Eq,
  {
    let idxs: Vec<_> = (0..self.constraints.len())
      .filter(|&idx| self.constraints[idx].variables().next().is_none())
      .collect();

    // short circuit on all constraints empty
    if idxs.len() == self.constraints.len() {
      self.constraints.clear();
      self.references.clear();
    }

    // remove constraints in reverse order
    // this avoids swapping constraints that'll be removed
    for idx in idxs.into_iter().rev() {
      self.remove_idx(idx);
    }
  }

  /// Minimises the overlap between constraints within this system.\
  /// This effectively removes duplicated assignments by constraints.
  ///
  /// ## Returns
  ///
  /// A mutable reference to allow method chaining
  pub fn minimise(&mut self) -> Result<&mut Self, C::ConflictErr>
  where
    C::Var: Hash + Eq,
  {
    // We need to get around the borrow checker hating holding references
    // to different items within slices / `Vec`s.
    // To do this we can create a placeholder item for "taking" items from `self.constraints`
    // invariant 1: `placeholder`` is **not** left in `self.constraints`
    // invariant 2: no methods of `placeholder` are called
    let mut placeholder: C = unsafe { MaybeUninit::zeroed().assume_init() };

    while let Some(idx) = self.to_minimise.pop_first() {
      let overlaps = self.overlaps_at(idx);

      // delete overlapping constraints from references before updating
      for &idx in &overlaps {
        for var in self.constraints[idx].variables() {
          self
            .references
            .get_mut(&var)
            .expect("Invariant 2: self.references should have back references for all variables")
            .remove(&idx);
        }
      }

      // reduce all overlapping constraints with the constraint at `idx`
      let constraint = mem::replace(&mut self.constraints[idx], placeholder);
      let reduced: Vec<_> = overlaps
        .iter()
        .filter_map(|&overlap| {
          // invariant 2 is maintained here as overlaps does not contain `idx`
          self.constraints[overlap]
            .reduce(&constraint)
            .map(|reduced| reduced.then_some(overlap))
            .transpose()
        })
        .collect::<Result<_, _>>()?;
      // maintain invariant 1, remove placeholder from `self.constraints`
      placeholder = mem::replace(&mut self.constraints[idx], constraint);

      // re-add overlapping constraints to references after update
      for &idx in &overlaps {
        for var in self.constraints[idx].variables() {
          self
            .references
            .get_mut(&var)
            .expect("Invariant 2 and we haven't deleted any map entries")
            .insert(idx);
        }
      }

      // add any constraints successfully reduced to minimise from
      self.to_minimise.extend(reduced);
    }

    Ok(self)
  }

  /// Returns the best constraint to explore, according to a given heuristic
  ///
  /// ## Arguments
  ///
  /// - `heur`: the heuristic to use, takes the constraint and all overlapping constraints\
  ///   and returns a key that can be used to rank the best constraint to explore
  ///
  /// ## Returns
  ///
  /// The best constraint to explore
  pub(self) fn best_constraint<H: Heuristic<C>>(&self, heuristic: &mut H) -> Option<&C>
  where
    C::Var: Hash + Eq,
  {
    self
      .constraints
      .iter()
      .enumerate()
      .map(|(idx, cons)| {
        (
          cons,
          self
            .overlaps_at(idx)
            .into_iter()
            .map(|idx| &self.constraints[idx])
            .collect::<Vec<_>>(),
        )
      })
      .max_by_key(|(cons, overlaps)| heuristic.rank(cons, overlaps))
      .map(|(cons, _score)| cons)
  }

  /// Finds the indexes of constraints that overlap the constraint at `idx`
  fn overlaps_at(&self, idx: usize) -> HashSet<usize>
  where
    C::Var: Hash + Eq,
  {
    let mut overlaps: HashSet<_> = self.constraints[idx]
      .variables()
      .filter_map(|var| self.references.get(&var))
      .flat_map(|idxs| idxs.iter().copied())
      .collect();

    overlaps.remove(&idx);
    overlaps
  }

  /// Returns all solutions to this system of equations,\
  /// using the default heuristic
  ///
  /// ## Returns
  ///
  /// An iterator over possible solutions to the [`System`]
  ///
  /// ## See also
  ///
  /// - [`System::solve_with`] for providing a heuristic value
  /// - [`System::solve_with_default`] for providing a heuristic type
  pub fn solve(self) -> SystemIter<C, DefaultHeuristic>
  where
    C: Hash + Eq + Clone,
    C::Var: Hash + Eq,
    C::Solution: Default,
  {
    self.solve_with(Default::default())
  }

  /// Returns all solutions to this system of equations,\
  /// using the provided heuristic to rank which constraints to explore first.
  ///
  /// ## Arguments
  ///
  /// - `heuristic`: the heuristic to use to decide which constraint to explore.\
  ///   This receives the constraint to rank and all overlapping constraints
  ///
  /// ## Returns
  ///
  /// An iterator over possible solutions to the [`System`]
  ///
  /// ## See also
  ///
  /// - [`System::solve`] for using the default heuristic
  /// - [`System::solve_with_default`] for providing a heuristic type
  pub fn solve_with<H>(mut self, heuristic: H) -> SystemIter<C, H>
  where
    C: Hash + Eq + Clone,
    C::Var: Hash + Eq,
    C::Solution: Default,
  {
    let Ok(solution) = self.pop_solution() else {
      return SystemIter {
        stack: vec![],
        heuristic,
      };
    };

    SystemIter {
      stack: vec![(self, solution)],
      heuristic,
    }
  }

  /// Returns all solutions to this system of equations,\
  /// using the provided heuristic type to rank which constraints to explore first.
  ///
  /// ## Returns
  ///
  /// An iterator over possible solutions to the [`System`]
  ///
  /// ## See also
  ///
  /// - [`System::solve`] for using the default heuristic
  /// - [`System::solve_with`] for providing a heuristic value
  pub fn solve_with_default<H: Default>(self) -> SystemIter<C, H>
  where
    C: Hash + Eq + Clone,
    C::Var: Hash + Eq,
    C::Solution: Default,
  {
    self.solve_with(Default::default())
  }
}

/// An iterator for all solutions to a given constraint system
///
/// @todo Symmetry breaking\
/// i.e. if reducing `constraint1` then `constraint2` leads to a contradiction\
/// don't attempt to reduce `constraint2` then `constraint1`.
///
/// @todo Parallelisation\
/// This'll mostly consist of working out how to split the solution iterator.\
/// This could be achieved my using a MaxHeap structure that we partition to split.
pub struct SystemIter<C: Constraint + Clone, H> {
  /// A stack of system and their current solutions
  stack: Vec<(System<C>, C::Solution)>,
  /// The heuristic used to decide which constraint to explore
  heuristic: H,
}

impl<C: Constraint + Clone, H: Heuristic<C>> Iterator for SystemIter<C, H>
where
  System<C>: Clone,
  C: Hash + Eq,
  C::Var: Hash + Eq,
  C::Solution: Default + Clone,
{
  type Item = C::Solution;
  fn next(&mut self) -> Option<Self::Item> {
    while let Some((system, solution)) = self.stack.pop() {
      // if we've reached a fully resolved solution, return it
      if system.is_empty() {
        return Some(solution);
      }

      // pick the best constraint to decompose and explore it
      let best = system
        .best_constraint(&mut self.heuristic)
        .expect("A non-empty System should have a best constraint");
      for decomposition in best.decompositions() {
        let mut new_sys = system.clone();
        new_sys.insert(decomposition);

        let Ok(new_sol) = new_sys.pop_solution() else {
          continue;
        };
        self.stack.push((new_sys, solution.clone().union(new_sol)));
      }
    }

    None
  }
}
