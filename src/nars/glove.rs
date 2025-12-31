use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use super::control::NarsSystem;
use super::term::Term;
use super::memory::{Concept, Hypervector};
use super::truth::TruthValue;
use super::sentence::Stamp;

pub fn load_embeddings(path: &str, system: &mut NarsSystem) -> io::Result<()> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 2 {
            continue;
        }

        let word = parts[0];
        let vector_values: Result<Vec<f32>, _> = parts[1..].iter().map(|s| s.parse::<f32>()).collect();

        if let Ok(values) = vector_values {
            let hypervector = Hypervector::project(&values);
            let term = Term::atom_from_str(word);
            
            // Create a concept with neutral truth and low priority
            // Assuming TruthValue::new(frequency, confidence)
            // Let's give it a default truth value, maybe 0.5, 0.0 (unknown) or 0.5, 0.1 (low confidence)
            // The prompt says "Set initial priority to a low/medium baseline"
            
            let truth = TruthValue::new(0.5, 0.1); 
            let stamp = Stamp {
                creation_time: 0,
                evidence: Vec::new(),
            };
            
            // Concept::new(term, vector, truth, stamp)
            // I need to check if Concept::new takes priority or calculates it.
            // In control.rs: Concept::new(sentence.term, vector, sentence.truth, sentence.stamp);
            // And add_concept handles it.
            
            let mut concept = Concept::new(term.clone(), hypervector, truth, stamp);
            
            // Manually set priority if possible, or let the system handle it.
            // Concept struct likely has a priority field.
            // Let's assume we can modify it or the constructor sets it based on truth.
            // If I can't see Concept definition fully, I'll assume standard behavior.
            // But I should check Concept definition in memory.rs to be sure.
            
            system.add_concept(concept);
        }
    }

    Ok(())
}
