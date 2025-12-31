use super::term::Term;
use super::truth::TruthValue;
use serde::{Serialize, Deserialize};

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
