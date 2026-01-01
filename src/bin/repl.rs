use anyhow::Result;
use hybrid_nars_rust::nars::control::NarsSystem;
use hybrid_nars_rust::nars::parser::parse_narsese;
use hybrid_nars_rust::nars::memory::{Concept, Hypervector};
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("Hybrid NARS Rust REPL");
    println!("Type Narsese input or 'exit' to quit.");

    // Increase similarity threshold to 0.55 to avoid matching random noise
    let mut system = NarsSystem::new(0.1, 0.55);

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
        } else if trimmed.starts_with(".export ") {
            let filename = trimmed[8..].trim();
            if filename.is_empty() {
                println!("Usage: .export <filename>");
                continue;
            }
            
            let file = match std::fs::File::create(filename) {
                Ok(f) => f,
                Err(e) => {
                    println!("Failed to create file: {}", e);
                    continue;
                }
            };
            let writer = std::io::BufWriter::new(file);
            
            let export_data: Vec<serde_json::Value> = system.memory.values().map(|concept| {
                let term_str = match &concept.term {
                    hybrid_nars_rust::nars::term::Term::Atom(s) => s.clone(),
                    _ => concept.term.to_display_string(),
                };
                
                serde_json::json!({
                    "term": term_str,
                    "usage": (concept.priority * 100.0) as u32, // Mock usage from priority
                    "vector": concept.vector.bits.to_vec()
                })
            }).collect();

            if let Err(e) = serde_json::to_writer(writer, &export_data) {
                println!("Failed to serialize memory: {}", e);
            } else {
                println!("Memory exported to {}", filename);
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        match parse_narsese(trimmed) {
            Ok(sentence) => {
                println!("Parsed: {:?}", sentence);
                let vector = Hypervector::from_term(&sentence.term);
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
