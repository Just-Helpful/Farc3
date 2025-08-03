use super::TileResult;
use super::TileState;
use std::ops::Deref;

pub fn print_states<R: Deref<Target = [TileState]>>(states: &[R]) {
  println!("+{}+", "-".repeat(states[0].len()));
  for row in states {
    print!("|");
    for state in row.deref() {
      if state.tiles.is_empty() {
        print!("_");
      } else {
        print!("{}", state.count);
      }
    }
    println!("|");
  }
  println!("+{}+", "-".repeat(states[0].len()));
}

pub fn print_results<R: Deref<Target = [TileResult]>>(states: &[R]) {
  println!("+{}+", "-".repeat(states[0].len()));
  for row in states {
    print!("|");
    for result in row.deref() {
      let chr = match result {
        &TileResult::Unknown => '_',
        &TileResult::Mine => 'M',
        &TileResult::Safe => 'S',
      };
      print!("{chr}");
    }
    println!("|");
  }
  println!("+{}+", "-".repeat(states[0].len()));
}
