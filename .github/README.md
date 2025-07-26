# Arc3 Constraint Solving

<!-- cargo-rdme start -->

[![Crate documentation](https://raw.githubusercontent.com/Just-Helpful/Farc3/refs/heads/main/.github/badges/docs.svg)](
  https://docs.rs/farc3/latest/farc3
)
[![Github homepage](https://raw.githubusercontent.com/Just-Helpful/Farc3/refs/heads/main/.github/badges/github.svg)](
  https://github.com/Just-Helpful/Farc3
)
[![Package version](https://raw.githubusercontent.com/Just-Helpful/Farc3/refs/heads/main/.github/badges/version.svg)](
  https://crates.io/crates/farc3
)
[![Package version](https://raw.githubusercontent.com/Just-Helpful/Farc3/refs/heads/main/.github/badges/coverage.svg)][coverage-url]

[coverage-url]: https://github.com/Just-Helpful/Farc3/actions/runs/16542221943/artifacts/3621669344

A semi-generic approach to solving Constraint Satisfaction Problems,\
with the possibility to optimise based on the implementation of Constraints.

The primary exports of this crate are:

- [`System`] for generic constraint solving
- the [`Assignment`] trait for assigning values to variables
- the [`Constraint`] trait for constraining the values variables can take
- the [`Heuristic`] trait for deciding search order for constraint solving

[`System`]: https://docs.rs/farc3/latest/farc3/system/struct.System.html
[`Assignment`]: https://docs.rs/farc3/latest/farc3/assignment/trait.Assignment.html
[`Constraint`]: https://docs.rs/farc3/latest/farc3/constraint/trait.Constraint.html
[`Heuristic`]: https://docs.rs/farc3/latest/farc3/heuristics/trait.Heuristic.html

There's also some common variants of constraints:

- `DiscreteConstraint` that covers most forms of discrete constraints
- [`MineConstraint`] that can be used for minesweeper mine solving

[`MineConstraint`]: https://docs.rs/farc3/latest/farc3/systems/mines/constraint/struct.MineConstraint.html

## Examples

```rust
use farc3::prelude::*;

// Construct the two mine constraints:
// 1. 2 mines among tiles 0, 1 and 2
// 2. 1 mine among tiles 1 and 2
let constraint_0 = MineConstraint::new([0, 1, 2], 2);
let constraint_1 = MineConstraint::new([1, 2], 1);

// Construct the constraint system from these 2 constraints
let mut sys = System::from([
  constraint_0,
  constraint_1
]);

// Find all solutions to the system
let sltns: HashSet<_> = sys.solve().collect();

// All solutions should mark tile 0 as a mine
// as only one of 1 and 2 can be a mine
assert_eq!(
  sltns,
  HashSet::from([
    MineAssignment::new(/*safe*/ [1], /*mines*/ [2, 0]),
    MineAssignment::new(/*safe*/ [2], /*mines*/ [1, 0]),
  ])
);
```

<!-- cargo-rdme end -->
