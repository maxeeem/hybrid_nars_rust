use std::collections::{HashMap, BinaryHeap};
use std::cmp::Ordering;
use super::term::Term;
use super::memory::{Concept, Hypervector};
use super::rules::{InferenceRule, TruthFunction, load_default_rules};
use super::rule_loader::load_rules;
use super::glove::load_embeddings;
use super::unify::{unify_with_bindings, Bindings};
use super::sentence::{Sentence, Punctuation, Stamp};
use super::truth::{TruthValue, revision};

#[derive(Debug)]
struct Task {
    concept_term: Term,
    priority: f32,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub struct NarsSystem {
    pub memory: HashMap<Term, Concept>,
    pub rules: Vec<InferenceRule>,
    buffer: BinaryHeap<Task>,
    learning_rate: f32,
    similarity_threshold: f32,
    pub output_buffer: Vec<Sentence>,
}

impl NarsSystem {
    pub fn new(learning_rate: f32, similarity_threshold: f32) -> Self {
        Self {
            memory: HashMap::new(),
            rules: load_default_rules(),
            buffer: BinaryHeap::new(),
            learning_rate,
            similarity_threshold,
            output_buffer: Vec::new(),
        }
    }

    pub fn input(&mut self, sentence: Sentence) {
        let vector = Hypervector::random();
        let concept = Concept::new(sentence.term, vector, sentence.truth, sentence.stamp);
        self.add_concept(concept);
    }

    pub fn add_concept(&mut self, concept: Concept) {
        if let Some(existing_concept) = self.memory.get_mut(&concept.term) {
            // Revision
            let revised_truth = revision(existing_concept.truth, concept.truth);
            existing_concept.truth = revised_truth;
            
            // Emit revised sentence
            let sentence = Sentence::new(existing_concept.term.clone(), Punctuation::Judgement, revised_truth, existing_concept.stamp.clone());
            self.output_buffer.push(sentence);

            let task = Task {
                concept_term: existing_concept.term.clone(),
                priority: existing_concept.priority,
            };
            self.buffer.push(task);
        } else {
            let task = Task {
                concept_term: concept.term.clone(),
                priority: concept.priority,
            };
            self.memory.insert(concept.term.clone(), concept);
            self.buffer.push(task);
        }
    }

    pub fn cycle(&mut self) {
        // Step 1: Selection
        if let Some(task) = self.buffer.pop() {
            let term_a = task.concept_term;
            
            // We need to clone concept_a to avoid borrowing issues with self.memory
            let concept_a = if let Some(c) = self.memory.get(&term_a) {
                c.clone()
            } else {
                return;
            };

            // Step 2: Association (HDC)
            // Before finding a match, try immediate inference on the selected task
            self.reason_single(&concept_a);

            let mut best_match_term: Option<Term> = None;
            let mut max_sim = -1.0;

            for (term_b, concept_b) in &self.memory {
                if *term_b == term_a {
                    continue;
                }
                let sim = concept_a.vector.similarity(&concept_b.vector);
                if sim > max_sim && sim > self.similarity_threshold {
                    max_sim = sim;
                    best_match_term = Some(term_b.clone());
                }
            }

            if let Some(term_b) = best_match_term {
                let concept_b = self.memory.get(&term_b).unwrap().clone();
                
                // Step 3: Reasoning
                self.reason(&concept_a, &concept_b);
                self.reason(&concept_b, &concept_a);

                // Step 5: Learning (Hebbian)
                // Update vectors in memory
                if let Some(c_a) = self.memory.get_mut(&term_a) {
                    c_a.vector.update(&concept_b.vector, self.learning_rate);
                }
                if let Some(c_b) = self.memory.get_mut(&term_b) {
                    c_b.vector.update(&concept_a.vector, self.learning_rate);
                }
            }
        }
    }

    fn reason(&mut self, concept_a: &Concept, concept_b: &Concept) {
        // Check for evidence overlap
        if has_evidence_overlap(&concept_a.stamp, &concept_b.stamp) {
            return;
        }

        // Collect applicable rules and bindings first to avoid borrowing self.rules while mutating self
        let mut inferences_to_execute = Vec::new();

        for (rule_idx, rule) in self.rules.iter().enumerate() {
            // Try to unify premises with (A, B)
            // Rule premises: [P1, P2]
            // We try P1 <-> A, P2 <-> B
            
            if rule.premises.len() != 2 {
                continue; 
            }

            // Try Unification
            // 1. Unify P1 with A
            if let Some(bindings_1) = unify_with_bindings(&rule.premises[0], &concept_a.term, HashMap::new()) {
                // 2. Unify P2 with B, using bindings from 1
                if let Some(final_bindings) = unify_with_bindings(&rule.premises[1], &concept_b.term, bindings_1) {
                    // Success!
                    inferences_to_execute.push((rule_idx, final_bindings));
                }
            }
        }

        // Execute inferences
        for (rule_idx, bindings) in inferences_to_execute {
            let rule = &self.rules[rule_idx];
            let conclusion = rule.conclusion.clone();
            
            if let TruthFunction::Double(tf) = rule.truth_fn {
                self.execute_inference_logic(conclusion, tf, &bindings, concept_a, concept_b);
            }
        }
    }

    fn reason_single(&mut self, concept: &Concept) {
        let mut inferences_to_execute = Vec::new();
        for (rule_idx, rule) in self.rules.iter().enumerate() {
            if rule.premises.len() != 1 { continue; }
            
            if let Some(bindings) = unify_with_bindings(&rule.premises[0], &concept.term, HashMap::new()) {
                inferences_to_execute.push((rule_idx, bindings));
            }
        }
        
        for (rule_idx, bindings) in inferences_to_execute {
            let rule = &self.rules[rule_idx];
            if let TruthFunction::Single(tf) = rule.truth_fn {
                self.execute_single_inference(rule.conclusion.clone(), tf, &bindings, concept);
            }
        }
    }

    fn execute_single_inference(&mut self, conclusion_template: Term, truth_fn: fn(TruthValue) -> TruthValue, bindings: &Bindings, concept: &Concept) {
        let conclusion_term = substitute(&conclusion_template, bindings);
        let new_truth = (truth_fn)(concept.truth);
        let new_stamp = concept.stamp.clone(); 
        
        // For immediate inference, we can reuse the vector or project it. 
        // Reusing it implies semantic similarity which is often true for conversion/contraposition.
        let new_vector = concept.vector.clone();

        let new_concept = Concept::new(conclusion_term.clone(), new_vector, new_truth, new_stamp.clone());
        
        let sentence = Sentence::new(conclusion_term, Punctuation::Judgement, new_truth, new_stamp);
        self.output_buffer.push(sentence);
        self.add_concept(new_concept);
    }

    fn execute_inference_logic(&mut self, conclusion_template: Term, truth_fn: fn(TruthValue, TruthValue) -> TruthValue, bindings: &Bindings, concept_a: &Concept, concept_b: &Concept) {
        // Generate conclusion term
        let conclusion_term = substitute(&conclusion_template, bindings);
        
        // Calculate Truth
        let new_truth = (truth_fn)(concept_a.truth, concept_b.truth);
        
        // Merge Stamps
        let new_stamp = merge_stamps(&concept_a.stamp, &concept_b.stamp);

        // Create new Concept
        let new_vector = Hypervector::bundle(&[concept_a.vector, concept_b.vector]);

        let new_concept = Concept::new(conclusion_term.clone(), new_vector, new_truth, new_stamp.clone());
        
        // Add to output buffer
        let sentence = Sentence::new(conclusion_term, Punctuation::Judgement, new_truth, new_stamp);
        self.output_buffer.push(sentence);
        
        // Add to system
        self.add_concept(new_concept);
    }

    pub fn load_rules_from_file(&mut self, path: &str) {
        let new_rules = load_rules(path);
        if !new_rules.is_empty() {
            println!("Loaded {} rules from {}", new_rules.len(), path);
            self.rules = new_rules;
        } else {
            println!("No rules loaded from {}, keeping defaults.", path);
        }
    }

    pub fn load_embeddings_from_file(&mut self, path: &str) -> std::io::Result<()> {
        load_embeddings(path, self)
    }
}

fn has_evidence_overlap(stamp1: &Stamp, stamp2: &Stamp) -> bool {
    for e1 in &stamp1.evidence {
        if stamp2.evidence.contains(e1) {
            return true;
        }
    }
    false
}

fn merge_stamps(stamp1: &Stamp, stamp2: &Stamp) -> Stamp {
    let mut new_evidence = stamp1.evidence.clone();
    for e in &stamp2.evidence {
        if !new_evidence.contains(e) {
            new_evidence.push(*e);
        }
    }
    // Sort for consistency if needed, but not strictly required for logic
    new_evidence.sort(); 
    
    Stamp {
        creation_time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        evidence: new_evidence,
    }
}

fn substitute(term: &Term, bindings: &Bindings) -> Term {
    match term {
        Term::Var(_, _) => {
            if let Some(val) = bindings.get(term) {
                val.clone()
            } else {
                term.clone()
            }
        },
        Term::Compound(op, args) => {
            let new_args = args.iter().map(|arg| substitute(arg, bindings)).collect();
            Term::Compound(op.clone(), new_args)
        },
        _ => term.clone(),
    }
}
