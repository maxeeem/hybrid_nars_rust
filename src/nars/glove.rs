use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter};
use std::path::Path;
use super::control::NarsSystem;
use super::term::Term;
use super::memory::{Concept, Hypervector, ProjectionMatrix};
use super::truth::TruthValue;
use super::sentence::Stamp;

pub fn load_embeddings(path: &str, system: &mut NarsSystem) -> io::Result<()> {
    let txt_path = Path::new(path);
    let bin_path = txt_path.with_extension("bin");

    // Try loading from binary cache first
    if bin_path.exists() {
        println!("Loading cached embeddings from {:?}...", bin_path);
        let file = File::open(&bin_path)?;
        let reader = BufReader::new(file);
        match bincode::deserialize_from::<_, Vec<Concept>>(reader) {
            Ok(concepts) => {
                println!("Loaded {} concepts from cache.", concepts.len());
                for concept in concepts {
                    system.add_concept(concept, false);
                }
                return Ok(());
            },
            Err(e) => {
                println!("Failed to load cache: {}. Falling back to text parsing.", e);
                // Fall through to text parsing
            }
        }
    }

    if !txt_path.exists() {
        return Ok(());
    }

    println!("Parsing embeddings from {:?}...", txt_path);
    let file = File::open(txt_path)?;
    let reader = BufReader::new(file);
    
    let mut concepts = Vec::new();
    let mut count = 0;
    let mut projection_matrix: Option<ProjectionMatrix> = None;
    
    // Limit to top 20,000 words for performance during demo
    // Full GloVe (400k words) would take hours to project on CPU
    let max_words = 20_000; 

    for line in reader.lines() {
        if count >= max_words {
            println!("Reached limit of {} words. Stopping.", max_words);
            break;
        }

        let line = line?;
        count += 1;
        if count % 100 == 0 {
            print!("\rProcessed {} lines...", count);
            use std::io::Write;
            io::stdout().flush()?;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.len() < 2 {
            continue;
        }

        let word = parts[0];
        let vector_values: Result<Vec<f32>, _> = parts[1..].iter().map(|s| s.parse::<f32>()).collect();

        if let Ok(values) = vector_values {
            // Initialize projection matrix on first valid vector
            if projection_matrix.is_none() {
                println!("Initializing projection matrix for dimension {}...", values.len());
                projection_matrix = Some(ProjectionMatrix::new(values.len()));
            }

            let hypervector = if let Some(ref matrix) = projection_matrix {
                Hypervector::project_with_matrix(&values, matrix)
            } else {
                Hypervector::project(&values) // Fallback, should not happen
            };

            let term = Term::atom_from_str(word);
            
            let truth = TruthValue::new(0.5, 0.1); 
            let stamp = Stamp {
                creation_time: 0,
                evidence: Vec::new(),
            };
            
            let concept = Concept::new(term, hypervector, truth, stamp);
            concepts.push(concept);
        }
    }

    // Save to cache
    println!("Saving cache to {:?}...", bin_path);
    if let Ok(file) = File::create(&bin_path) {
        let writer = BufWriter::new(file);
        if let Err(e) = bincode::serialize_into(writer, &concepts) {
            println!("Failed to save cache: {}", e);
        }
    }

    // Add to system
    for concept in concepts {
        system.add_concept(concept, false);
    }

    Ok(())
}
