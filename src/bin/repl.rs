use anyhow::Result;
use hybrid_nars_rust::nars::control::NarsSystem;
use hybrid_nars_rust::nars::parser::parse_narsese;
use hybrid_nars_rust::nars::memory::{Concept, Hypervector};
use hybrid_nars_rust::nars::term::{Term, Operator};
use hybrid_nars_rust::nars::sentence::{Sentence, Punctuation, Stamp};
use hybrid_nars_rust::nars::truth::TruthValue;
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
        } else if trimmed.starts_with(".save ") {
            let filename = trimmed[6..].trim();
            if filename.is_empty() {
                println!("Usage: .save <filename>");
                continue;
            }
            if let Err(e) = system.save_memory(filename) {
                println!("Failed to save memory: {}", e);
            } else {
                println!("Memory saved to {}", filename);
            }
            continue;
        } else if trimmed.starts_with(".load ") {
            let filename = trimmed[6..].trim();
            if filename.is_empty() {
                println!("Usage: .load <filename>");
                continue;
            }
            if let Err(e) = system.load_memory(filename) {
                println!("Failed to load memory: {}", e);
            } else {
                println!("Memory loaded from {}", filename);
            }
            continue;
        } else if trimmed.starts_with(".drift ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() != 3 {
                println!("Usage: .drift <term1> <term2>");
                continue;
            }
            let t1_str = parts[1];
            let t2_str = parts[2];
            let term1 = Term::Atom(t1_str.to_string());
            let term2 = Term::Atom(t2_str.to_string());

            // Helper to get vector
            let get_vector = |sys: &NarsSystem, t: &Term| -> Hypervector {
                if let Some(c) = sys.memory.get(t) {
                    c.vector
                } else {
                    Hypervector::from_term(t)
                }
            };

            let v1_initial = get_vector(&system, &term1);
            let v2_initial = get_vector(&system, &term2);
            let sim_initial = v1_initial.similarity(&v2_initial);

            println!("Initial Similarity({}, {}): {:.4}", t1_str, t2_str, sim_initial);

            let stmt = format!("<{} --> {}>.", t1_str, t2_str);
            println!("Injecting: {}", stmt);
            match parse_narsese(&stmt) {
                Ok(sentence) => {
                    system.input(sentence);
                    
                    // Activate the terms themselves to facilitate interaction
                    if let Some(mut c1) = system.memory.get(&term1).cloned() {
                        c1.priority = 0.99; // Boost priority
                        system.add_concept(c1, false);
                    }

                    if let Some(mut c2) = system.memory.get(&term2).cloned() {
                        c2.priority = 0.99; // Boost priority
                        system.add_concept(c2, false);
                    }

                    println!("Running 20 cycles...");
                    for _ in 0..20 {
                        system.cycle();
                    }
                    
                    let v1_final = get_vector(&system, &term1);
                    let v2_final = get_vector(&system, &term2);
                    let sim_final = v1_final.similarity(&v2_final);
                    let delta = sim_final - sim_initial;

                    println!("Final Similarity: {:.4}", sim_final);
                    println!("Delta: {:.4}", delta);
                    
                    if delta > 0.0 {
                        println!("SUCCESS: Concept Drift Detected.");
                    } else {
                        println!("FAIL: No Drift Detected.");
                    }
                },
                Err(e) => println!("Error parsing injection: {:?}", e),
            }
            continue;
        } else if trimmed.starts_with(".drift_transitive ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() != 4 {
                println!("Usage: .drift_transitive <A> <B> <C>");
                continue;
            }
            let a_str = parts[1];
            let b_str = parts[2];
            let c_str = parts[3];
            
            let term_a = Term::Atom(a_str.to_string());
            let term_b = Term::Atom(b_str.to_string());
            let term_c = Term::Atom(c_str.to_string());

            // Helper to get vector
            let get_vector = |sys: &NarsSystem, t: &Term| -> Hypervector {
                if let Some(c) = sys.memory.get(t) {
                    c.vector
                } else {
                    Hypervector::from_term(t)
                }
            };

            // Measure Sim(A, C)
            let v_a_initial = get_vector(&system, &term_a);
            let v_c_initial = get_vector(&system, &term_c);
            let sim_initial = v_a_initial.similarity(&v_c_initial);
            println!("Initial Sim({}, {}): {:.4}", a_str, c_str, sim_initial);

            // Input <A --> B>
            let stmt1 = Term::Compound(Operator::Inheritance, vec![term_a.clone(), term_b.clone()]);
            let sent1 = Sentence::new(stmt1, Punctuation::Judgement, TruthValue::new(1.0, 0.9), Stamp::new(0, vec![]));
            system.input(sent1);
            println!("Input: <{} --> {}>", a_str, b_str);

            // Input <B --> C>
            let stmt2 = Term::Compound(Operator::Inheritance, vec![term_b.clone(), term_c.clone()]);
            let sent2 = Sentence::new(stmt2, Punctuation::Judgement, TruthValue::new(1.0, 0.9), Stamp::new(0, vec![]));
            system.input(sent2);
            println!("Input: <{} --> {}>", b_str, c_str);

            // Run 50 cycles
            println!("Running 50 cycles...");
            for _ in 0..50 {
                system.cycle();
            }

            // Measure Sim(A, C)
            let v_a_final = get_vector(&system, &term_a);
            let v_c_final = get_vector(&system, &term_c);
            let sim_final = v_a_final.similarity(&v_c_final);
            println!("Final Sim({}, {}): {:.4}", a_str, c_str, sim_final);

            let drift = sim_final - sim_initial;
            println!("Drift: {:.4}", drift);
            if drift > 0.0 {
                println!("SUCCESS: {} moved towards {}", a_str, c_str);
            } else {
                println!("FAILURE: {} did not move towards {}", a_str, c_str);
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        match parse_narsese(trimmed) {
            Ok(sentence) => {
                println!("Parsed: {:?}", sentence);
                
                // Run the input
                system.input(sentence.clone());

                // Run inference cycles
                print!("Thinking...");
                io::stdout().flush().unwrap();
                for _ in 0..10 {
                    system.cycle();
                    print!(".");
                    io::stdout().flush().unwrap();
                }
                println!();

                // IF it was a Question, look for the answer
                if sentence.punctuation == Punctuation::Question {
                    if let Some(answer) = system.answer_query(&sentence.term) {
                        println!("Answer: {} %{:.2};{:.2}%", 
                            answer.term.to_display_string(), 
                            answer.truth.frequency, 
                            answer.truth.confidence
                        );
                    } else {
                        println!("Answer: I don't know.");
                    }
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
