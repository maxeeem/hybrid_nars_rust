use anyhow::Result;
use hybrid_nars_rust::nars::control::NarsSystem;
use hybrid_nars_rust::nars::parser::parse_narsese;
use hybrid_nars_rust::nars::memory::{Concept, Hypervector};
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("Hybrid NARS Rust REPL");
    println!("Type Narsese input or 'exit' to quit.");

    let mut system = NarsSystem::new(0.1, 0.5);

    // Load rules
    let rules_path = "assets/rules.lisp";
    if std::path::Path::new(rules_path).exists() {
        system.load_rules_from_file(rules_path);
    } else {
        println!("Warning: {} not found, using default rules.", rules_path);
    }

    // Load embeddings
    let glove_path = "assets/glove.txt";
    if std::path::Path::new(glove_path).exists() {
        println!("Loading embeddings from {}...", glove_path);
        if let Err(e) = system.load_embeddings_from_file(glove_path) {
            println!("Failed to load embeddings: {}", e);
        } else {
            println!("Embeddings loaded.");
        }
    }

    loop {
        print!(">> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed == "exit" {
            break;
        } else if trimmed == ".rules" {
            println!("Loaded Rules: {}", system.rules.len());
            continue;
        } else if trimmed == ".stats" {
            println!("Concepts in Memory: {}", system.memory.len());
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        match parse_narsese(trimmed) {
            Ok((_, sentence)) => {
                println!("Parsed: {:?}", sentence);
                let vector = Hypervector::random();
                let concept = Concept::new(sentence.term, vector, sentence.truth, sentence.stamp);
                system.add_concept(concept);

                println!("Running 5 cycles...");
                for _ in 0..5 {
                    system.cycle();
                }
                
                // Print top concepts in memory (simple debug view)
                println!("Memory Size: {}", system.memory.len());
            },
            Err(e) => {
                println!("Parse Error: {:?}", e);
            }
        }
    }

    Ok(())
}
