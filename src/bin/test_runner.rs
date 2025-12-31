use anyhow::{Context, Result};
use hybrid_nars_rust::nars::control::NarsSystem;
use hybrid_nars_rust::nars::parser::parse_narsese;
use hybrid_nars_rust::nars::sentence::Sentence;
use hybrid_nars_rust::nars::term::Term;
use hybrid_nars_rust::nars::truth::TruthValue;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: test_runner <path_to_nal_file_or_directory>");
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);

    if path.is_dir() {
        let mut paths: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().map_or(false, |ext| ext == "nal"))
            .collect();
        
        // Sort for consistent order
        paths.sort();

        let mut failures = 0;
        let mut total = 0;

        for p in paths {
            println!("Running test: {:?}", p.file_name().unwrap());
            if let Err(e) = run_test_file(&p) {
                eprintln!("Test failed: {:?} - {}", p, e);
                failures += 1;
            }
            total += 1;
            println!("----------------------------------------");
        }
        
        println!("Summary: {}/{} tests passed.", total - failures, total);
        if failures > 0 {
            std::process::exit(1);
        }
    } else {
        run_test_file(path)?;
        println!("Test passed: {:?}", path);
    }

    Ok(())
}

fn run_test_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let file = File::open(path).context("Failed to open test file")?;
    let reader = BufReader::new(file);
    
    let mut system = NarsSystem::new(0.1, 0.5);
    
    // Load rules
    let rules_path = "assets/rules.lisp";
    if std::path::Path::new(rules_path).exists() {
        system.load_rules_from_file(rules_path);
    }

    // Load embeddings
    let glove_path = "assets/glove.txt";
    if std::path::Path::new(glove_path).exists() {
        let _ = system.load_embeddings_from_file(glove_path);
    }

    let mut active_expectations: Vec<String> = Vec::new();
    let mut accumulated_outputs: Vec<Sentence> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // 1. Output Expectation
        if trimmed.starts_with("''outputMustContain") {
            if let Some(start) = trimmed.find("('") {
                if let Some(end) = trimmed.rfind("')") {
                    let expected = &trimmed[start+2..end];
                    active_expectations.push(expected.to_string());
                    check_expectations(&accumulated_outputs, &mut active_expectations)?;
                }
            }
            continue;
        }
        
        if trimmed.starts_with("'") {
            // Comment
            continue;
        }

        // 2. Cycle Step (Integer)
        if let Ok(steps) = trimmed.parse::<usize>() {
            for _ in 0..steps {
                system.cycle();
                accumulated_outputs.append(&mut system.output_buffer);
                check_expectations(&accumulated_outputs, &mut active_expectations)?;
            }
            continue;
        }

        // 3. Narsese Input
        match parse_narsese(trimmed) {
            Ok(sentence) => {
                system.input(sentence);
                accumulated_outputs.append(&mut system.output_buffer);
            },
            Err(_) => {
                // Log warning but continue
            }
        }
        
        check_expectations(&accumulated_outputs, &mut active_expectations)?;
    }
    
    if !active_expectations.is_empty() {
        return Err(anyhow::anyhow!("Unmet expectations: {:?}", active_expectations));
    }

    Ok(())
}

fn check_expectations(outputs: &[Sentence], expectations: &mut Vec<String>) -> Result<()> {
    if expectations.is_empty() {
        return Ok(());
    }

    let mut matched_indices = Vec::new();
    
    for (i, expected_str) in expectations.iter().enumerate() {
        match parse_narsese(expected_str) {
            Ok(expected_sentence) => {
                for output in outputs {
                    if terms_match(&output.term, &expected_sentence.term) {
                        if truth_matches(output.truth, expected_sentence.truth) {
                            matched_indices.push(i);
                            break; 
                        }
                    }
                }
            },
            Err(e) => {
                eprintln!("Warning: Could not parse expectation '{}': {}", expected_str, e);
            }
        }
    }
    
    matched_indices.sort_by(|a, b| b.cmp(a));
    matched_indices.dedup();
    
    for i in matched_indices {
        expectations.remove(i);
    }
    
    Ok(())
}

fn terms_match(t1: &Term, t2: &Term) -> bool {
    t1 == t2
}

fn truth_matches(t1: TruthValue, t2: TruthValue) -> bool {
    let epsilon = 0.01;
    (t1.frequency - t2.frequency).abs() < epsilon && (t1.confidence - t2.confidence).abs() < epsilon
}
