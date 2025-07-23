/// Unit testing generic [`Assignments`]
///
/// [`Assignments`]: crate::systems::generic::assignment::GenericAssignment
mod assignments {
  use super::super::assignment::DiscreteAssignment;
  use crate::prelude::Assignment;
  use std::collections::HashMap;

  #[test]
  fn construction() {
    DiscreteAssignment::from_iter([("a", 1), ("b", 2), ("c", 1)]);
  }

  #[test]
  fn iteration() {
    let assign = DiscreteAssignment::from_iter([("a", 1), ("b", 2), ("c", 1)]);
    let var_map: HashMap<&str, usize> = HashMap::from_iter(assign);
    assert_eq!(var_map.get("a"), Some(&1));
    assert_eq!(var_map.get("b"), Some(&2));
    assert_eq!(var_map.get("c"), Some(&1));
  }

  #[test]
  fn intersection() {
    let assign0 = DiscreteAssignment::from_iter([("a", 1), ("b", 2), ("c", 1)]);
    let assign1 = DiscreteAssignment::from_iter([/*     */ ("b", 2), ("c", 2), ("d", 1)]);
    let assign = assign0.intersection(assign1);

    let var_map: HashMap<&str, usize> = HashMap::from_iter(assign);
    assert_eq!(var_map.get("a"), None);
    assert_eq!(var_map.get("b"), Some(&2));
    assert_eq!(var_map.get("c"), None); // contradiction
    assert_eq!(var_map.get("d"), None);
  }

  #[test]
  fn union() {
    let assign0 = DiscreteAssignment::from_iter([("a", 1), ("b", 2), ("c", 1)]);
    let assign1 = DiscreteAssignment::from_iter([/*     */ ("b", 2), ("c", 2), ("d", 1)]);
    let assign = assign0.union(assign1);

    let var_map: HashMap<&str, usize> = HashMap::from_iter(assign);
    assert_eq!(var_map.get("a"), Some(&1));
    assert_eq!(var_map.get("b"), Some(&2));
    assert_eq!(var_map.get("c"), None); // contradiction
    assert_eq!(var_map.get("d"), Some(&1));
  }
}

/// Unit testing generic [`Constraints`]
///
/// [`Constraints`]: crate::systems::generic::assignment::GenericConstraint
mod constraints {
  use crate::prelude::{Constraint, DiscreteConstraint};
  use std::collections::{HashMap, HashSet};

  #[test]
  fn construction() {
    DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)],
      [("a", false), ("b", true), ("c", true)],
    ]);
  }

  #[test]
  #[should_panic]
  fn inconsistent() {
    DiscreteConstraint::from_iter([
      vec![("a", true), ("b", false), ("c", true)],
      vec![("a", false), ("b", true), ("d", true)],
    ]);
  }

  #[test]
  fn size() {
    let cons = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)],
      [("a", false), ("b", true), ("c", true)],
      [("a", false), ("b", true), ("c", true)], // duplicate assignment
    ]);
    assert_eq!(cons.size(), 2)
  }

  #[test]
  fn variables() {
    let cons = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)],
      [("a", false), ("b", true), ("c", true)],
    ]);
    assert_eq!(
      HashSet::from_iter(cons.variables()),
      HashSet::from(["a", "b", "c"])
    );
  }

  #[test]
  fn decompositions() {
    let cons = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)],
      [("a", false), ("b", true), ("c", true)],
    ]);
    let vars = HashSet::from(["a", "b", "c"]);

    for decomp in cons.decompositions() {
      assert_eq!(decomp.size(), 1);
      let vars0 = decomp.variables().collect();
      assert!(vars.is_superset(&vars0));
    }
  }

  #[test]
  fn pop_solution() {
    let mut cons = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)],
      [("a", false), ("b", true), ("c", true)],
    ]);

    let sltn = cons.pop_solution().unwrap();
    assert_eq!(
      HashMap::from_iter(sltn), //
      HashMap::from([("c", true)])
    );

    assert_eq!(cons.size(), 2);
    assert_eq!(
      HashSet::from_iter(cons.variables()),
      HashSet::from(["a", "b"])
    );
  }

  #[test]
  fn pop_solution_all() {
    let mut cons = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)], //
    ]);

    let sltn = cons.pop_solution().unwrap();
    assert_eq!(
      HashMap::from_iter(sltn),
      HashMap::from([("a", true), ("b", false), ("c", true)])
    );

    assert_eq!(cons.size(), 1);
    assert_eq!(HashSet::from_iter(cons.variables()), HashSet::new());
  }

  #[test]
  fn reduce() {
    let mut cons0 = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)],
      [("a", false), ("b", true), ("c", true)],
    ]);
    let cons1 = DiscreteConstraint::from_iter([
      [("a", true), ("b", true), ("c", false)],
      [("a", true), ("b", false), ("c", true)],
    ]);

    cons0.reduce(&cons1).unwrap();
    assert_eq!(cons0.size(), 1);
    assert_eq!(
      HashSet::from_iter(cons0.variables()),
      HashSet::from(["a", "b", "c"])
    );

    let sltn = cons0.pop_solution().unwrap();
    assert_eq!(
      HashMap::from_iter(sltn),
      HashMap::from([("a", true), ("b", false), ("c", true)])
    );

    assert_eq!(cons0.size(), 1);
    assert_eq!(HashSet::from_iter(cons0.variables()), HashSet::new());
  }

  #[test]
  fn conflicts() {
    let mut cons0 = DiscreteConstraint::from_iter([
      [("a", true), ("b", true), ("c", false)],
      [("a", true), ("b", false), ("c", true)],
    ]);
    let cons1 = DiscreteConstraint::from_iter([
      [("a", false), ("b", true)], //
      [("a", false), ("b", false)],
    ]);

    let res = cons0.reduce(&cons1);
    assert_eq!(res, Err(()));
  }
}

/// Testing generic constraint compatability with [`System`] solving
///
/// [`System`]: crate::system::System
mod solver {
  use std::collections::HashMap;

  use super::super::constraint::DiscreteConstraint;
  use crate::prelude::System;

  #[test]
  fn unresolvable() {
    let cons = DiscreteConstraint::from_iter([
      [("a", true), ("b", false)], //
      [("a", false), ("b", true)],
    ]);

    let mut sys = System::from_iter([cons]);
    sys.minimise().unwrap();
    assert_eq!(sys.len(), 1);

    let sltn = sys.pop_solution().unwrap();
    assert_eq!(HashMap::from_iter(sltn), HashMap::from([]));
    assert_eq!(sys.len(), 1);
  }

  #[test]
  fn trivial() {
    let cons = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)], //
    ]);

    let mut sys = System::from_iter([cons]);
    sys.minimise().unwrap();
    assert_eq!(sys.len(), 1);

    let sltn = sys.pop_solution().unwrap();
    assert_eq!(
      HashMap::from_iter(sltn),
      HashMap::from([("a", true), ("b", false), ("c", true)])
    );
    assert_eq!(sys.len(), 0);
  }

  /// The solver can fully solve a system of constraints
  #[test]
  fn full_minimise() {
    let cons0 = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true)],
      [("a", false), ("b", true), ("c", true)],
    ]);
    let cons1 = DiscreteConstraint::from_iter([
      [("a", true), ("b", true), ("c", false)],
      [("a", true), ("b", false), ("c", true)],
    ]);

    let mut sys = System::from_iter([cons0, cons1]);
    sys.minimise().unwrap();
    assert_eq!(sys.len(), 2);

    let sltn = sys.pop_solution().unwrap();
    assert_eq!(
      HashMap::from_iter(sltn),
      HashMap::from([("a", true), ("b", false), ("c", true)])
    );
    assert_eq!(sys.len(), 0);
  }

  /// The solver can pick out the fully resolved part of a system
  #[test]
  fn partial_minimise() {
    let cons0 = DiscreteConstraint::from_iter([
      [("a", true), ("b", false), ("c", true), ("d", false)],
      [("a", true), ("b", false), ("c", true), ("d", true)],
      [("a", false), ("b", true), ("c", true), ("d", true)],
    ]);
    let cons1 = DiscreteConstraint::from_iter([
      [("a", true), ("b", true), ("c", false)],
      [("a", true), ("b", false), ("c", true)],
    ]);

    let mut sys = System::from_iter([cons0, cons1]);
    sys.minimise().unwrap();
    assert_eq!(sys.len(), 2);

    let sltn = sys.pop_solution().unwrap();
    assert_eq!(
      HashMap::from_iter(sltn),
      HashMap::from([("a", true), ("b", false), ("c", true)])
    );

    // we have a remaining undecided constraint
    assert_eq!(sys.len(), 1);
    assert_eq!(
      Vec::from_iter(sys),
      vec![DiscreteConstraint::from_iter([
        [("d", false)],
        [("d", true)]
      ])]
    );
  }
}
