use cruciverbal_external::GameFile;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game = GameFile::load("sample_puzzle.json")?;
    println!("Successfully loaded: {}", game.crossword.title);
    println!("Author: {}", game.crossword.author);
    println!("Grid size: {}x{}", game.crossword.grid.width, game.crossword.grid.height);
    println!("Number of clues: {}", game.crossword.clues.len());
    Ok(())
}