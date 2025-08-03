use std::collections::HashSet;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default, Clone)]
pub struct TileState {
  /// positions this constraint affects
  tiles: HashSet<[usize; 2]>,
  /// The number of mines in `tiles`
  count: usize,
}

mod print;
use print::{print_results, print_states};

/// Pops a single item from a hashset
fn pop_set<T: Clone + Hash + Eq>(set: &mut HashSet<T>) -> Option<T> {
  let item = set.iter().next()?.clone();
  set.remove(&item);
  Some(item)
}

#[derive(Debug, PartialEq, Clone)]
pub enum TileResult {
  Unknown,
  Mine,
  Safe,
}

/// Extracts all tile results from the minesweeper board\
/// This mutates the board state, removing all tiles resolved.
fn extract_results<R: DerefMut<Target = [TileState]>>(
  board: &mut [R],
) -> Option<Vec<Vec<TileResult>>> {
  let mut results: Vec<_> = board
    .iter()
    .map(|row| vec![TileResult::Unknown; row.len()])
    .collect();

  let tile_idxs = board.iter_mut().flat_map(|row| row.iter_mut());

  for tile in tile_idxs {
    let result = if tile.count == 0 {
      TileResult::Safe
    } else if tile.count == tile.tiles.len() {
      TileResult::Mine
    } else {
      continue;
    };

    // remove tilestate
    tile.count = 0;
    tile.tiles = HashSet::new();

    for &[i0, j0] in &tile.tiles {
      if results[j0][i0] == TileResult::Unknown {
        results[j0][i0] = result.clone();
        continue;
      }

      if results[j0][i0] != result {
        return None;
      }
    }
  }

  Some(results)
}

/// Assigns all mine tiles that can be known to either `Mine` or `Safe`
///
/// ## Arguments
///
/// - board: the constraints representing the board to be solved
/// - start: where to start constraint solving from
///
/// ## Returns
///
/// Whether the board had conflicting constraints
pub fn resolve_tiles<R: DerefMut<Target = [TileState]>>(
  board: &mut [R],
  start: impl IntoIterator<Item = [usize; 2]>,
) -> Option<Vec<Vec<TileResult>>> {
  let mut updates = HashSet::from_iter(start);
  let mut tile = TileState::default();

  while let Some([i, j]) = pop_set(&mut updates) {
    println!("{i}, {j}");

    // pull the tile out (this is **only** safe in single threads)
    // @note we need to do this to make rust's borrow checker happy
    // @todo when multithreading, we'll need a mutex to lock access
    std::mem::swap(&mut tile, &mut board[j][i]);
    println!("{tile:?}");

    // perform reduction with all affected tiles
    for [i0, j0] in tile.tiles.iter().copied().collect::<Vec<_>>() {
      // skip if we've already removed this neighbour
      if !tile.tiles.contains(&[i0, j0]) {
        continue;
      }

      // reduce with tile0
      let tile0 = &board[j0][i0];
      println!("Candidate {tile0:?}");

      if tile0.tiles.is_empty() {
        println!("  Empty state, skipping");
        continue;
      }

      let mut tiles = tile0.tiles.clone();
      tiles.remove(&[i, j]);
      println!("  Produced tiles {tiles:?}");

      let diff: HashSet<_> = tile.tiles.difference(&tiles).copied().collect();

      // all safe case
      if tile0.count == 0 {
        println!("  All tiles safe, reducing");
        // contradictions!
        if diff.len() < tile.count {
          return None;
        }

        tile.tiles = diff;
        updates.insert([i0, j0]);
        continue;
      }

      // all mine case
      if tile0.count == tile0.tiles.len() {
        println!("  All tiles mine, reducing");
        // contradictions!
        if tile.count < tile0.count {
          return None;
        }
        if diff.len() < tile.count - tile0.count {
          return None;
        }

        tile.tiles = diff;
        tile.count -= tile0.count;
        updates.insert([i0, j0]);
        continue;
      }

      // superset case
      if tile.tiles.is_superset(&tiles) {
        println!("  Tiles are a subset, reducing");
        // contradictions!
        if tile.count < tile0.count {
          return None;
        }
        if diff.len() < tile.count - tile0.count {
          return None;
        }

        tile.tiles = diff;
        tile.count -= tile0.count;
        updates.insert([i0, j0]);
        continue;
      }

      println!("  No action taken");
    }

    // put the tile back after we've successfully reduced it
    std::mem::swap(&mut tile, &mut board[j][i]);
  }

  extract_results(board)
}

mod solver;

/// Finds all the neighbours of a given index in a grid size
pub fn neighbours_of([i, j]: [usize; 2], [w, h]: [usize; 2]) -> impl Iterator<Item = [usize; 2]> {
  let mut nbs = Vec::with_capacity(8);

  if 0 < j {
    if 0 < i {
      nbs.push([i - 1, j - 1])
    }
    nbs.push([i, j - 1]);
    if i < w - 1 {
      nbs.push([i + 1, j - 1])
    }
  }

  if 0 < i {
    nbs.push([i - 1, j])
  }
  if i < w - 1 {
    nbs.push([i + 1, j])
  }

  if j < h - 1 {
    if 0 < i {
      nbs.push([i - 1, j + 1])
    }
    nbs.push([i, j + 1]);
    if i < w - 1 {
      nbs.push([i + 1, j + 1])
    }
  }

  nbs.into_iter()
}

pub fn to_board_state<R: Deref<Target = [bool]>>(mines: &[R]) -> Vec<Vec<TileState>> {
  if mines.len() == 0 {
    return vec![];
  }
  let [w, h] = [mines[0].len(), mines.len()];

  (0..h)
    .map(|j0| {
      (0..w)
        .map(|i0| {
          if mines[j0][i0] {
            return TileState {
              tiles: HashSet::new(),
              count: 0,
            };
          }

          let mut count = 0;
          let tiles: HashSet<_> = neighbours_of([i0, j0], [w, h]).collect();

          for &[i, j] in &tiles {
            if mines[j][i] {
              count += 1;
            }
          }

          TileState { tiles, count }
        })
        .collect()
    })
    .collect()
}

fn main() {
  let board = vec![
    vec![true, false, true],
    vec![true, false, false],
    vec![false, false, true],
  ];

  let states = to_board_state(&board);
  print_states(&states);

  let mut states0 = states.clone();
  let res = resolve_tiles(&mut states0, [[1, 0], [1, 1]]).unwrap();
  print_results(&res);
}
