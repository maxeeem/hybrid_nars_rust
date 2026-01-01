use std::collections::HashMap;
use std::cmp::Ordering;
use std::fs::File;
use std::error::Error;
use super::term::{Term, Operator};
use super::memory::{Concept, Hypervector, ConceptStore};
use super::bag::Bag;
use super::rules::{InferenceRule, TruthFunction};
use super::static_rules::get_all_rules;
use super::glove::load_embeddings;
use super::unify::{unify_with_bindings, Bindings};
use super::sentence::{Sentence, Punctuation, Stamp};
use super::truth::{TruthValue, revision};

pub struct NarsSystem {
    pub memory: ConceptStore,
    pub rules: Vec<InferenceRule>,
    pub buffer: Bag<Term>,
    pub learning_rate: f32,
    pub similarity_threshold: f32,
    pub output_buffer: Vec<Sentence>,
}

impl NarsSystem {
    pub fn new(learning_rate: f32, similarity_threshold: f32) -> Self {
        let rules = get_all_rules();
        Self {
            memory: ConceptStore::new(10000),
            rules,
            buffer: Bag::new(100),
            learning_rate,
            similarity_threshold,
            output_buffer: Vec::new(),
        }
    }

    pub fn resolve_vector(&self, term: &Term) -> Hypervector {
        if let Some(concept) = self.memory.get(term) {
            return concept.vector;
        }
        match term {
            Term::Compound(op, args) => {
                let arg_vectors: Vec<Hypervector> = args.iter().map(|a| self.resolve_vector(a)).collect();
                Hypervector::compound(op, &arg_vectors)
            },
            _ => Hypervector::from_term(term),
        }
    }

    pub fn input(&mut self, sentence: Sentence) {
        let vector = self.resolve_vector(&sentence.term);
        let concept = Concept::new(sentence.term, vector, sentence.truth, sentence.stamp);
        self.add_concept(concept, sentence.punctuation == Punctuation::Judgement);
    }

    pub fn add_concept(&mut self, mut concept: Concept, is_judgement: bool) {
        let existing_concept_opt = self.memory.get(&concept.term).cloned();

        if let Some(mut existing_concept) = existing_concept_opt {
             if is_judgement {
                 let revised_truth = revision(existing_concept.truth, concept.truth);
                 existing_concept.truth = revised_truth;
                 let belief = Sentence::new(concept.term.clone(), Punctuation::Judgement, concept.truth, concept.stamp.clone());
                 existing_concept.add_belief(belief);
                 let sent = Sentence::new(existing_concept.term.clone(), Punctuation::Judgement, revised_truth, existing_concept.stamp.clone());
                 self.output_buffer.push(sent);
             }
             self.memory.put(existing_concept.clone());
             
             let priority = (existing_concept.priority * existing_concept.durability).clamp(0.01, 0.99);
             self.buffer.put(existing_concept.term.clone(), priority);
        } else {
             if is_judgement {
                 let belief = Sentence::new(concept.term.clone(), Punctuation::Judgement, concept.truth, concept.stamp.clone());
                 concept.add_belief(belief);
             }
             self.memory.put(concept.clone());
             let priority = (concept.priority * concept.durability).clamp(0.01, 0.99);
             self.buffer.put(concept.term.clone(), priority);
        }
        
        // Vector Learning Logic
        if is_judgement {
            if let Term::Compound(Operator::Inheritance, args) = &concept.term {
                if args.len() == 2 {
                    let subject_term = &args[0];
                    let predicate_term = &args[1];
                    
                    let p_vector = self.resolve_vector(predicate_term);
                    
                    let subject_term = subject_term.clone();
                    
                    let mut s_concept = if let Some(c) = self.memory.get(&subject_term) {
                        c.clone()
                    } else {
                        let vector = Hypervector::from_term(&subject_term);
                        Concept::new(subject_term.clone(), vector, TruthValue::new(0.5, 0.0), Stamp::new(0, vec![]))
                    };
                    
                    s_concept.vector.update(&p_vector, self.learning_rate);
                    self.memory.put(s_concept);
                }
            }
        }
    }

    pub fn cycle(&mut self) {
        // 1. Selection (Probabilistic from Bag)
        let term_a = match self.buffer.take() {
            Some(t) => t,
            None => return,
        };
        
        // Retrieve Concept A
        let concept_a = match self.memory.get(&term_a) {
            Some(c) => c.clone(),
            None => return,
        };

        // 2. Association (Random Sampling for AIKR)
        // We cannot scan all memory. We take a sample of keys.
        let sample_size = 20;
        let partners: Vec<Term> = self.memory.keys()
            .take(sample_size * 3) // Grab a chunk (HashMap order is pseudo-random)
            .filter(|t| **t != term_a)
            .take(sample_size)
            .cloned()
            .collect();

        // 3. Geometric Attention ("The Pull")
        for term_b in partners {
            if let Some(concept_b) = self.memory.get(&term_b) {
                let sim = concept_a.vector.similarity(&concept_b.vector);
                
                if sim >= self.similarity_threshold {
                    // Activate B (Pull into Attention)
                    // If A is active, and A~B, then B becomes active.
                    let new_p = (sim * 0.9).clamp(0.01, 0.99);
                    self.buffer.put(term_b.clone(), new_p);
                    
                    // Reason
                    // Cloning to satisfy borrow checker
                    let cb = concept_b.clone();
                    self.reason(&concept_a, &cb);
                    self.reason(&cb, &concept_a);
                    
                    // Hebbian Learning
                    if let Some(c_a) = self.memory.get_mut(&term_a) {
                        c_a.vector.update(&cb.vector, self.learning_rate);
                    }
                    if let Some(c_b) = self.memory.get_mut(&term_b) {
                        c_b.vector.update(&concept_a.vector, self.learning_rate);
                    }
                }
            }
        }
        
        self.reason_single(&concept_a);
    }

    fn reason(&mut self, concept_a: &Concept, concept_b: &Concept) {
        // println!("Inside reason function");
        // Check for evidence overlap
        if concept_a.stamp.overlaps(&concept_b.stamp) {
            // println!("Overlap detected!");
            return;
        }

        // Collect applicable rules and bindings first to avoid borrowing self.rules while mutating self
        let mut inferences_to_execute = Vec::new();

        // println!("Rules count: {}", self.rules.len());

        for (rule_idx, rule) in self.rules.iter().enumerate() {
            // Try to unify premises with (A, B)
            // Rule premises: [P1, P2]
            // We try P1 <-> A, P2 <-> B
            
            // println!("Rule {} premises: {}", rule_idx, rule.premises.len());

            if rule.premises.len() != 2 {
                continue; 
            }

            // Debug unification
            // println!("Trying rule {} P1 with A: {:?}", rule_idx, concept_a.term);

            // Try Unification
            // 1. Unify P1 with A
            if let Some(bindings_1) = unify_with_bindings(&rule.premises[0], &concept_a.term, HashMap::new()) {
                // println!("  P1 matched! Bindings: {:?}", bindings_1);
                // 2. Unify P2 with B, using bindings from 1
                if let Some(final_bindings) = unify_with_bindings(&rule.premises[1], &concept_b.term, bindings_1) {
                    // println!("  Rule {} ({}) matched! Executing inference.", rule_idx, rule.name);
                    // Success!
                    inferences_to_execute.push((rule_idx, final_bindings));
                } else {
                    // println!("  P2 failed to match B: {:?}", concept_b.term);
                }
            } else {
                // println!("  P1 failed to match A: {:?}", concept_a.term);
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
                // println!("  Single Rule {} ({}) matched! Executing inference.", rule_idx, rule.name); // Added debug print
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
        
        // Debug Output
        println!("[DEBUG] Derived: {:?} %{};{}%", conclusion_term, new_truth.frequency, new_truth.confidence);

        // For immediate inference, we can reuse the vector or project it. 
        // Reusing it implies semantic similarity which is often true for conversion/contraposition.
        let new_vector = concept.vector.clone();

        let new_concept = Concept::new(conclusion_term.clone(), new_vector, new_truth, new_stamp.clone());
        
        let sentence = Sentence::new(conclusion_term, Punctuation::Judgement, new_truth, new_stamp);
        self.output_buffer.push(sentence);
        self.add_concept(new_concept, true);
    }

    fn execute_inference_logic(&mut self, conclusion_template: Term, truth_fn: fn(TruthValue, TruthValue) -> TruthValue, bindings: &Bindings, concept_a: &Concept, concept_b: &Concept) {
        // Generate conclusion term
        let conclusion_term = substitute(&conclusion_template, bindings);
        
        // Calculate Truth
        let new_truth = (truth_fn)(concept_a.truth, concept_b.truth);
        
        // Merge Stamps
        let new_stamp = concept_a.stamp.merge(&concept_b.stamp);

        // Debug Output
        println!("[DEBUG] Derived: {:?} %{};{}%", conclusion_term, new_truth.frequency, new_truth.confidence);

        // Create new Concept
        let new_vector = Hypervector::bundle(&[concept_a.vector, concept_b.vector]);

        let new_concept = Concept::new(conclusion_term.clone(), new_vector, new_truth, new_stamp.clone());
        
        // Add to output buffer
        let sentence = Sentence::new(conclusion_term, Punctuation::Judgement, new_truth, new_stamp);
        self.output_buffer.push(sentence);
        
        // Add to system
        self.add_concept(new_concept, true);
    }


    pub fn load_embeddings_from_file(&mut self, path: &str) -> std::io::Result<()> {
        load_embeddings(path, self)
    }

    pub fn save_memory(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let f = File::create(filename)?;
        bincode::serialize_into(f, &self.memory)?;
        Ok(())
    }

    pub fn load_memory(&mut self, filename: &str) -> Result<(), Box<dyn Error>> {
        let f = File::open(filename)?;
        let mut store: ConceptStore = bincode::deserialize_from(f)?;
        // Rebuild bag
        for (term, concept) in store.map.iter() {
             let utility = (concept.priority * concept.durability).clamp(0.01, 0.99);
             store.priority_bag.put(term.clone(), utility);
        }
        self.memory = store;
        Ok(())
    }

    pub fn answer_query(&self, term: &Term) -> Option<Sentence> {
        if let Some(concept) = self.memory.get(term) {
            // Only return beliefs with actual confidence
            return concept.beliefs.iter()
                .filter(|b| b.truth.confidence > 0.01)
                .max_by(|a, b| a.truth.confidence.partial_cmp(&b.truth.confidence).unwrap())
                .cloned();
        }
        None
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
