use super::term::Term;
use super::truth::TruthValue;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Punctuation {
    Judgement, // .
    Question,  // ?
    Goal,      // !
    Quest,     // @
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stamp {
    pub creation_time: u64,
    pub evidence: Vec<u64>, 
}

impl Stamp {
    pub fn new(creation_time: u64, evidence: Vec<u64>) -> Self {
        Self {
            creation_time,
            evidence,
        }
    }

    pub fn overlaps(&self, other: &Stamp) -> bool {
        for id in &self.evidence {
            if other.evidence.contains(id) {
                return true;
            }
        }
        false
    }

    pub fn merge(&self, other: &Stamp) -> Stamp {
        let mut new_evidence = self.evidence.clone();
        for id in &other.evidence {
            if !new_evidence.contains(id) {
                new_evidence.push(*id);
            }
        }
        
        // Prune oldest IDs if length exceeds limit
        let limit = 100;
        if new_evidence.len() > limit {
            let overflow = new_evidence.len() - limit;
            new_evidence.drain(0..overflow);
        }

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Stamp {
            creation_time: current_time,
            evidence: new_evidence,
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sentence {
    pub term: Term,
    pub punctuation: Punctuation,
    pub truth: TruthValue, 
    pub stamp: Stamp,
}

impl Sentence {
    pub fn new(term: Term, punctuation: Punctuation, truth: TruthValue, stamp: Stamp) -> Self {
        Self {
            term,
            punctuation,
            truth,
            stamp,
        }
    }
}
