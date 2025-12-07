/// Example demonstrates reading a puzzle (downloaded from: https://club72.wordpress.com/2025/12/05/puzzle-1148-freestyle-1069/).
///
/// > cargo run --example sample_puzzle
fn main() {
    let puzzle =
        puz_parse::parse_file("providers/examples/data/Puzzle1148Freestyle1069.puz").unwrap();

    println!("Title: {}", puzzle.info.title);
    println!("Author: {}", puzzle.info.author);
    println!("Copyright: {}", puzzle.info.copyright);
    println!("Dimensions: {}x{}", puzzle.info.width, puzzle.info.height);
    println!("Version: {}", puzzle.info.version);
    assert_eq!(puzzle.info.width, puzzle.grid.blank.len() as u8);

    println!("Number of clues (a): {}", puzzle.clues.across.len());
    println!("Number of clues (d): {}", puzzle.clues.down.len());

    println!("Grid:");
    for row_i in 0..puzzle.info.height as usize {
        let row = &puzzle.grid.blank[row_i];
        for ch in row.chars() {
            print!(
                "{}",
                if ch == '-' {
                    "_" /* empty */
                } else {
                    "■" /* filled */
                }
            );
        }
        println!("");
    }

    println!("\nSolution:");
    for row_i in 0..puzzle.info.height as usize {
        let row = &puzzle.grid.solution[row_i];
        for ch in row.chars() {
            print!("{ch}");
        }
        println!("");
    }

    println!("\nAcross clues:");
    for (i, clue) in puzzle.clues.across.iter() {
        println!("{}: {}", i, clue);
    }
    println!("\nDown clues:");
    for (i, clue) in puzzle.clues.down.iter() {
        println!("{}: {}", i, clue);
    }
}
