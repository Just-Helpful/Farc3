//! Errors produced when mine constraints conflict

/// The error that is produced when 2 mine constraints conflict.
///
/// ## Example
///
/// ```
/// # use farc3::constraint::Constraint;
/// # use farc3::systems::mines::{
/// #   constraint::MineConstraint,
/// #   errors::MineConflicts
/// # };
///
/// let mut cons0 = MineConstraint::new([0, 1], 1);
/// let cons1 = MineConstraint::new([0, 1], 2);
///
/// let res = cons0.reduce(&cons1);
/// assert_eq!(res, Err(MineConflicts));
/// ```
///
/// @todo provide better debug info on mine conflicts
#[derive(Debug, PartialEq, Eq)]
pub struct MineConflicts;
