use super::TileResult;
use super::TileState;

/// An iterator over possible solutions to a given board
pub struct BoardSolutions {
  pub(crate) stack: Vec<(Vec<Vec<TileState>>, Vec<Vec<TileResult>>)>,
}

impl From<Vec<Vec<TileState>>> for BoardSolutions {
  fn from(value: Vec<Vec<TileState>>) -> Self {
    let result: Vec<_> = value
      .iter()
      .map(|row| vec![TileResult::Unknown; row.len()])
      .collect();
    Self {
      stack: vec![(value, result)],
    }
  }
}

impl Iterator for BoardSolutions {
  type Item = Vec<Vec<TileResult>>;
  fn next(&mut self) -> Option<Self::Item> {
    while let Some((board, result)) = self.stack.pop() {
      // board exhausted, return solution
      if board
        .iter()
        .all(|row| row.iter().all(|tile| tile.tiles.len() == 0))
      {
        return Some(result);
      }

      // Build backlinks for affected variables
      let mut links: Vec<_> = board.iter().map(|row| vec![vec![]; row.len()]).collect();
      for ([i0, j0], tile) in board
        .iter()
        .enumerate()
        .flat_map(|(j, row)| row.iter().enumerate().map(move |(i, tile)| ([i, j], tile)))
      {
        for &[i, j] in &tile.tiles {
          links[j][i].push([i0, j0]);
        }
      }

      // Positions to explore that aren't solved
      let candidates: Vec<_> = board
        .iter()
        .flat_map(|row| {
          row.iter().filter_map(|tile| {
            if tile.count == 0 {
              return None;
            }
            if tile.count == tile.tiles.len() {
              return None;
            }
            Some(tile.clone())
          })
        })
        .collect();

      // Find tile with min solutions and max affected
      let best = candidates.into_iter().min_by_key(|tile| {
        let n_solutions = (tile.tiles.len() + 1..=tile.count).product::<usize>()
          / (2..=(tile.count - tile.tiles.len())).product::<usize>();

        let n_affected = tile
          .tiles
          .iter()
          .map(|&[i, j]| links[j][i].len() as isize)
          .sum::<isize>();

        (n_solutions, -n_affected)
      })?;

      // Extend with possible solutions to this tile
      for &[i0, j0] in best.tiles.iter() {
        // assignments to safe tiles
        let mut n_board = board.clone();
        let mut n_result = result.clone();

        for &[i, j] in &links[j0][i0] {
          n_board[j][i].tiles.remove(&[i0, j0]);
        }
        n_result[j0][i0] = TileResult::Safe;

        self.stack.push((n_board, n_result))
      }

      for &[i0, j0] in best.tiles.iter() {
        // assignments to mine tiles
        let mut n_board = board.clone();
        let mut n_result = result.clone();

        for &[i, j] in &links[j0][i0] {
          n_board[j][i].tiles.remove(&[i0, j0]);
          n_board[j][i].count -= 1;
        }
        n_result[j0][i0] = TileResult::Mine;

        self.stack.push((n_board, n_result))
      }
    }

    None
  }
}
