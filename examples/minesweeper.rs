use std::collections::HashSet;
use std::hash::Hash;

#[derive(Default, Clone)]
pub struct TileState {
  /// positions this constraint affects
  tiles: HashSet<[usize; 2]>,
  /// The number of mines in `tiles`
  count: usize,
}

/// Pops a single item from a hashset
fn pop_set<T: Clone + Hash + Eq>(set: &mut HashSet<T>) -> Option<T> {
  let item = set.iter().next()?.clone();
  set.remove(&item);
  Some(item)
}

#[derive(PartialEq, Clone)]
pub enum TileResult {
  Unknown,
  Mine,
  Safe,
}

/// Extracts all tile results from the minesweeper board\
/// This mutates the board state, removing all tiles resolved.
fn extract_results(board: &mut [&mut [TileState]]) -> Option<Vec<Vec<TileResult>>> {
  let mut results: Vec<_> = board
    .iter()
    .map(|row| vec![TileResult::Unknown; row.len()])
    .collect();

  let tile_idxs = board.iter().flat_map(|row| row.iter());

  for tile in tile_idxs {
    let result = if tile.count == 0 {
      TileResult::Safe
    } else if tile.count == tile.tiles.len() {
      TileResult::Mine
    } else {
      continue;
    };

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
fn resolve_tiles(
  board: &mut [&mut [TileState]],
  start: impl IntoIterator<Item = [usize; 2]>,
) -> Option<Vec<Vec<TileResult>>> {
  let mut updates = HashSet::from_iter(start);
  let mut tile = TileState::default();

  while let Some([i, j]) = pop_set(&mut updates) {
    // pull the tile out (this is **only** safe in single threads)
    // @note we need to do this to make rust's borrow checker happy
    // @todo when multithreading, we'll need a mutex to lock access
    std::mem::swap(&mut tile, &mut board[j][i]);

    // perform reduction with all affected tiles
    for [i0, j0] in tile.tiles.iter().copied().collect::<Vec<_>>() {
      // skip if we've already removed this neighbour
      if tile.tiles.contains(&[i0, j0]) {
        continue;
      }

      // reduce with tile0
      let tile0 = &board[j0][i0];
      let diff: HashSet<_> = tile.tiles.difference(&tile0.tiles).copied().collect();

      // all safe case
      if tile0.count == 0 {
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
      if tile.tiles.is_subset(&tile0.tiles) {
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
    }

    // put the tile back after we've successfully reduced it
    std::mem::swap(&mut tile, &mut board[j][i]);
  }

  extract_results(board)
}

/** Solves a minesweeper board
 *
 * ## Arguments
 *
 * - board: the constraints representing the board to be solved
 * - start: where to start solving from
 *
 * ## Returns
 *
 * Whether there were conflicting constraints on the board
 */
pub fn solve_board(
  board: &mut [&mut [TileState]],
  start: impl IntoIterator<Item = [usize; 2]>,
) -> Option<Vec<Vec<TileResult>>> {
  resolve_tiles(board, start)?;

  // Positions to explore that aren't solved
  let candidates: Vec<_> = board
    .iter()
    .enumerate()
    .flat_map(|(j, row)| {
      row.iter().enumerate().filter_map(move |(i, tile)| {
        if tile.count == 0 {
          return None;
        }
        if tile.count == tile.tiles.len() {
          return None;
        }
        Some(([i, j], tile.clone()))
      })
    })
    .collect();

  // Build counts of affected variables
  let mut counts: Vec<_> = board.iter().map(|row| vec![0; row.len()]).collect();
  for tile in board.iter().flat_map(|row| row.iter()) {
    for &[i, j] in &tile.tiles {
      counts[j][i] += 1;
    }
  }

  // Find tile with min solutions and max affected
  let (idx, best) = candidates.into_iter().min_by_key(|(idx, tile)| {
    let n_solutions = (tile.tiles.len() + 1..=tile.count).product::<usize>()
      / (2..=(tile.count - tile.tiles.len())).product::<usize>();

    let n_affected = tile.tiles.iter().map(|&[i, j]| counts[j][i]).sum::<isize>();

    (n_solutions, -n_affected)
  })?;

  // Recurse on possible solutions to this tile
  let res = best
    .tiles
    .iter()
    .flat_map(|idx| {
      [TileResult::Mine, TileResult::Safe]
        .into_iter()
        .map(move |res| (*idx, res))
    })
    .map(|(idx, res)| {});

  None
}

/// Finds all the neighbours of a given index in a grid size
fn neighbours_of([i, j]: [usize; 2], [w, h]: [usize; 2]) -> impl Iterator<Item = [usize; 2]> {
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

fn main() {}
