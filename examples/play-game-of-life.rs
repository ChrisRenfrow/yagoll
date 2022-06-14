use std::{env, path::Path, thread, time};

use game_of_life::Board;

fn main() {
    let path = env::args()
        .nth(1)
        .expect("No path to game of life file provided!");
    let path = Path::new(&path[..]);
    assert!(path.exists() && path.is_file());
    println!("Path: {}", path.display());

    let num_cycles: usize = env::args()
        .nth(2)
        .expect("Please supply the number of cycles you'd like to simulate")
        .parse()
        .expect("Please supply a valid number for number of cycles");
    println!("Number of cycles: {}", num_cycles);

    let delay: u64 = env::args()
        .nth(3)
        .unwrap_or_else(|| "1000".to_string())
        .parse()
        .unwrap();
    println!("Delay in ms: {}", delay);

    let mut board = Board::new_from_file(path);
    println!("Board from {}:\n{}", path.display(), board);

    (0..num_cycles + 1).for_each(|i| {
        println!("Cycle: {}/{}\n{}", i, num_cycles, board);
        board.advance_cycle();
        thread::sleep(time::Duration::from_millis(delay));
    });
}
