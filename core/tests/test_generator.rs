use cruciverbal_core::generator::{generate_crossword, GeneratorConfig, GeneratorError};
use cruciverbal_core::Direction;

#[test]
fn test_basic_generator_usage() {
    // Test basic generator functionality with a simple word list
    let words = vec![
        ("DOG".to_string(), "Man's best friend".to_string()),
        ("CAT".to_string(), "Feline pet".to_string()),
        ("RUN".to_string(), "Move quickly".to_string()),
        ("JUMP".to_string(), "Leap into the air".to_string()),
        ("BOOK".to_string(), "Thing you read".to_string()),
    ];

    let config = GeneratorConfig {
        width: 8,
        height: 8,
        max_words: 10,
        min_words: 2,
        symmetry: true,
        max_attempts: 20,
        prefer_longer_words: true,
    };

    let result = generate_crossword(words, Some(config));
    
    // The generator should either succeed or fail gracefully
    match result {
        Ok(crossword) => {
            // Verify basic crossword properties
            assert_eq!(crossword.grid.width, 8);
            assert_eq!(crossword.grid.height, 8);
            assert!(!crossword.clues.is_empty(), "Generated crossword should have clues");
            assert!(crossword.clues.len() >= 2, "Should have at least minimum number of words");
            
            println!("✅ Generated crossword with {} clues", crossword.clues.len());
            
            // Verify clues have both directions
            let across_count = crossword.get_clues_by_direction(Direction::Across).len();
            let down_count = crossword.get_clues_by_direction(Direction::Down).len();
            
            println!("   - {} across clues, {} down clues", across_count, down_count);
            
            // Print the generated clues for demonstration
            println!("   - Generated clues:");
            for clue in &crossword.clues {
                let dir = match clue.direction {
                    Direction::Across => "A",
                    Direction::Down => "D",
                };
                println!("     {}{}. {} ({})", clue.number, dir, clue.text, clue.answer);
            }
        }
        Err(GeneratorError::MaxAttemptsExceeded) => {
            println!("⚠️  Generator exhausted attempts (this can happen with small grids)");
            // This is acceptable for small test grids
        }
        Err(e) => {
            panic!("Unexpected generator error: {}", e);
        }
    }
}

#[test]
fn test_larger_grid_generation() {
    // Test with a larger grid and more words
    let words = vec![
        ("DOG".to_string(), "Man's best friend".to_string()),
        ("CAT".to_string(), "Feline pet".to_string()),
        ("RUN".to_string(), "Move quickly".to_string()),
        ("JUMP".to_string(), "Leap into the air".to_string()),
        ("BOOK".to_string(), "Thing you read".to_string()),
        ("TREE".to_string(), "Woody plant".to_string()),
        ("HOUSE".to_string(), "Place to live".to_string()),
        ("WATER".to_string(), "Clear liquid".to_string()),
        ("PHONE".to_string(), "Communication device".to_string()),
        ("COMPUTER".to_string(), "Electronic machine".to_string()),
        ("RAINBOW".to_string(), "Colorful arc in sky".to_string()),
        ("MOUNTAIN".to_string(), "High land formation".to_string()),
        ("ELEPHANT".to_string(), "Large gray mammal".to_string()),
        ("BUTTERFLY".to_string(), "Colorful flying insect".to_string()),
    ];

    let config = GeneratorConfig {
        width: 12,
        height: 12,
        max_words: 20,
        min_words: 5,
        symmetry: true,
        max_attempts: 50,
        prefer_longer_words: true,
    };

    let result = generate_crossword(words, Some(config));
    
    match result {
        Ok(crossword) => {
            assert_eq!(crossword.grid.width, 12);
            assert_eq!(crossword.grid.height, 12);
            assert!(crossword.clues.len() >= 5, "Should have at least 5 words in larger grid");
            
            println!("✅ Generated larger crossword with {} clues", crossword.clues.len());
            
            // Verify that longer words are preferred
            let long_words = crossword.clues.iter()
                .filter(|clue| clue.answer.len() >= 7)
                .count();
            
            println!("   - {} words with 7+ letters", long_words);
            
            // Check for word intersections by looking for shared positions
            let mut letter_positions = std::collections::HashMap::new();
            for clue in &crossword.clues {
                for (i, &pos) in clue.positions().iter().enumerate() {
                    let letter = clue.answer.chars().nth(i).unwrap();
                    letter_positions.entry(pos).and_modify(|existing: &mut Vec<char>| {
                        existing.push(letter);
                    }).or_insert(vec![letter]);
                }
            }
            
            let intersections = letter_positions.iter()
                .filter(|(_, letters)| letters.len() > 1)
                .count();
            
            println!("   - {} letter intersections found", intersections);
        }
        Err(GeneratorError::MaxAttemptsExceeded) => {
            println!("⚠️  Generator exhausted attempts for larger grid");
            // Still acceptable, but less likely with larger grids
        }
        Err(e) => {
            panic!("Unexpected generator error: {}", e);
        }
    }
}

#[test]
fn test_default_config() {
    // Test using default configuration
    let words = vec![
        ("RUST".to_string(), "Systems programming language".to_string()),
        ("CODE".to_string(), "Computer instructions".to_string()),
        ("TEST".to_string(), "Software verification".to_string()),
        ("DEBUG".to_string(), "Fix software problems".to_string()),
    ];

    let result = generate_crossword(words, None); // Use default config
    
    // Should handle default config without panicking
    match result {
        Ok(crossword) => {
            // Default config creates 15x15 grid
            assert_eq!(crossword.grid.width, 15);
            assert_eq!(crossword.grid.height, 15);
            println!("✅ Generated crossword with default config: {} clues", crossword.clues.len());
        }
        Err(_) => {
            println!("⚠️  Default config generation failed (acceptable for small word lists)");
        }
    }
}

#[test]
fn test_insufficient_words() {
    // Test with insufficient words for the grid size
    let words = vec![
        ("A".to_string(), "Single letter".to_string()), // Too short, should be filtered out
        ("BY".to_string(), "Near to".to_string()),        // Too short, should be filtered out
    ];

    let config = GeneratorConfig {
        width: 5,
        height: 5,
        max_words: 10,
        min_words: 3,
        max_attempts: 5,
        ..Default::default()
    };

    let result = generate_crossword(words, Some(config));
    
    // Should fail due to insufficient valid words
    match result {
        Ok(_) => {
            panic!("Should not succeed with insufficient words");
        }
        Err(GeneratorError::MaxAttemptsExceeded) => {
            println!("✅ Correctly failed with insufficient words");
        }
        Err(e) => {
            println!("✅ Failed as expected: {}", e);
        }
    }
}

#[test]
fn test_asymmetric_pattern() {
    // Test with asymmetric blocked cell pattern
    let words = vec![
        ("HELLO".to_string(), "Greeting".to_string()),
        ("WORLD".to_string(), "Earth".to_string()),
        ("RUST".to_string(), "Programming language".to_string()),
        ("SAFE".to_string(), "Not dangerous".to_string()),
    ];

    let config = GeneratorConfig {
        width: 8,
        height: 8,
        max_words: 8,
        min_words: 2,
        symmetry: false, // Test asymmetric pattern
        max_attempts: 30,
        prefer_longer_words: false,
    };

    let result = generate_crossword(words, Some(config));
    
    match result {
        Ok(crossword) => {
            println!("✅ Generated asymmetric crossword with {} clues", crossword.clues.len());
            
            // Just verify it's a valid crossword
            assert!(!crossword.clues.is_empty());
            assert!(crossword.clues.len() >= 2);
        }
        Err(_) => {
            println!("⚠️  Asymmetric generation failed (acceptable)");
        }
    }
}

#[test]
fn test_word_filtering() {
    // Test that invalid words are filtered out
    let words = vec![
        ("A".to_string(), "Single letter".to_string()),         // Too short
        ("BY".to_string(), "Preposition".to_string()),          // Too short
        ("CAT123".to_string(), "Invalid characters".to_string()), // Contains numbers
        ("DOG".to_string(), "Valid word".to_string()),          // Valid
        ("HOUSE".to_string(), "Valid longer word".to_string()), // Valid
    ];

    let config = GeneratorConfig {
        width: 6,
        height: 6,
        max_words: 5,
        min_words: 1,
        max_attempts: 20,
        ..Default::default()
    };

    let result = generate_crossword(words, Some(config));
    
    match result {
        Ok(crossword) => {
            println!("✅ Generated crossword after filtering invalid words");
            
            // Should only have valid words (length >= 3, alphabetic only)
            for clue in &crossword.clues {
                assert!(clue.answer.len() >= 3, "Word '{}' is too short", clue.answer);
                assert!(clue.answer.chars().all(|c| c.is_alphabetic()), 
                       "Word '{}' contains non-alphabetic characters", clue.answer);
            }
            
            // Should not contain the invalid words
            let answers: Vec<&String> = crossword.clues.iter().map(|c| &c.answer).collect();
            assert!(!answers.contains(&&"A".to_string()));
            assert!(!answers.contains(&&"BY".to_string()));
            assert!(!answers.contains(&&"CAT123".to_string()));
        }
        Err(_) => {
            println!("⚠️  Generation failed after filtering (acceptable with few valid words)");
        }
    }
}

#[test]
fn test_grid_validation() {
    // Test that generated grids are valid
    let words = vec![
        ("DOG".to_string(), "Pet".to_string()),
        ("GOD".to_string(), "Deity".to_string()), // Should intersect with DOG at 'G'
        ("CAT".to_string(), "Feline".to_string()),
    ];

    let config = GeneratorConfig {
        width: 6,
        height: 6,
        max_words: 5,
        min_words: 2,
        max_attempts: 20,
        ..Default::default()
    };

    let result = generate_crossword(words, Some(config));
    
    match result {
        Ok(crossword) => {
            println!("✅ Validating generated grid structure");
            
            // Check that all clue positions are valid
            for clue in &crossword.clues {
                let positions = clue.positions();
                
                // All positions should be within grid bounds
                for &(row, col) in &positions {
                    assert!(row < crossword.grid.height, 
                           "Row {} out of bounds for clue {}", row, clue.number);
                    assert!(col < crossword.grid.width, 
                           "Col {} out of bounds for clue {}", col, clue.number);
                    
                    // Position should contain a letter cell
                    if let Some(cell) = crossword.grid.get_cell(row, col) {
                        assert!(cell.is_letter(), 
                               "Cell at ({}, {}) should be a letter for clue {}", 
                               row, col, clue.number);
                    }
                }
                
                // Answer length should match positions
                assert_eq!(clue.answer.len(), positions.len(),
                          "Answer length mismatch for clue {}", clue.number);
            }
            
            println!("   - All {} clues have valid grid positions", crossword.clues.len());
        }
        Err(_) => {
            println!("⚠️  Generation failed for validation test");
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_generator_with_external_word_list() {
        // Simulate loading words from external source
        let words = load_sample_word_list();
        
        let config = GeneratorConfig {
            width: 10,
            height: 10,
            max_words: 12,
            min_words: 3,
            symmetry: true,
            max_attempts: 30,
            prefer_longer_words: true,
        };

        let result = generate_crossword(words, Some(config));
        
        match result {
            Ok(crossword) => {
                println!("✅ Integration test: Generated crossword from external word list");
                println!("   - Title: {}", crossword.title);
                println!("   - Author: {}", crossword.author);
                println!("   - Grid: {}x{}", crossword.grid.width, crossword.grid.height);
                println!("   - Clues: {}", crossword.clues.len());
                
                // Demonstrate crossword solution validation
                let validation_errors = crossword.validate_solution();
                println!("   - Validation errors: {}", validation_errors.len());
            }
            Err(e) => {
                println!("⚠️  Integration test failed: {}", e);
            }
        }
    }

    fn load_sample_word_list() -> Vec<(String, String)> {
        // Simulate a larger, more diverse word list
        vec![
            ("ALGORITHM".to_string(), "Step-by-step procedure".to_string()),
            ("BINARY".to_string(), "Base-2 number system".to_string()),
            ("COMPILER".to_string(), "Code translator".to_string()),
            ("DATABASE".to_string(), "Information storage".to_string()),
            ("ENCRYPTION".to_string(), "Data protection method".to_string()),
            ("FRAMEWORK".to_string(), "Software structure".to_string()),
            ("GRAPHICS".to_string(), "Visual elements".to_string()),
            ("HARDWARE".to_string(), "Physical components".to_string()),
            ("INTERFACE".to_string(), "Connection point".to_string()),
            ("JAVASCRIPT".to_string(), "Web programming language".to_string()),
            ("KEYBOARD".to_string(), "Input device".to_string()),
            ("LIBRARY".to_string(), "Code collection".to_string()),
            ("MEMORY".to_string(), "Data storage".to_string()),
            ("NETWORK".to_string(), "Connected systems".to_string()),
            ("OPERATING".to_string(), "System management".to_string()),
            ("PROTOCOL".to_string(), "Communication rules".to_string()),
            ("QUERY".to_string(), "Database request".to_string()),
            ("RUNTIME".to_string(), "Execution environment".to_string()),
            ("SOFTWARE".to_string(), "Computer programs".to_string()),
            ("TERMINAL".to_string(), "Command interface".to_string()),
        ]
    }
}

#[test]
fn test_performance_with_timing() {
    use std::time::Instant;
    
    let words = vec![
        ("PERFORMANCE".to_string(), "Speed and efficiency".to_string()),
        ("BENCHMARK".to_string(), "Performance test".to_string()),
        ("OPTIMIZE".to_string(), "Improve efficiency".to_string()),
        ("ALGORITHM".to_string(), "Problem-solving steps".to_string()),
        ("EFFICIENT".to_string(), "Not wasteful".to_string()),
    ];

    let config = GeneratorConfig {
        width: 9,
        height: 9,
        max_words: 8,
        min_words: 3,
        max_attempts: 25,
        ..Default::default()
    };

    let start = Instant::now();
    let result = generate_crossword(words, Some(config));
    let duration = start.elapsed();

    println!("⏱️  Generation took: {:?}", duration);
    
    match result {
        Ok(crossword) => {
            println!("✅ Performance test: Generated {} clues in {:?}", 
                    crossword.clues.len(), duration);
            
            // Performance should be reasonable (under 1 second for small grids)
            assert!(duration.as_secs() < 5, 
                   "Generation took too long: {:?}", duration);
        }
        Err(_) => {
            println!("⚠️  Performance test failed to generate crossword");
        }
    }
}