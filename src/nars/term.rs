use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

// Deterministic hash function (FNV-1a)
pub fn deterministic_hash(s: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash = hash ^ (byte as u64);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VarType {
    Independent, // $
    Dependent,   // #
    Query,       // ?
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operator {
    Inheritance,      // -->
    Implication,      // ==>
    Similarity,       // <->
    Equivalence,      // <=>
    Instance,         // {--
    Property,         // --]
    InstanceProperty, // {-]
    Product,          // *
    ExtIntersection,  // |
    IntIntersection,  // &
    Difference,       // -
    DifferenceInt,    // ~
    Union,            // +
    ExtSet,           // {}
    IntSet,           // []
    Negation,         // --
    Conjunction,      // &&
    Disjunction,      // ||
    ExtImage,         // /
    IntImage,         // \
    ConcurrentImplication, // =|>
    PredictiveImplication, // =/>
    RetrospectiveImplication, // =\>
    ConcurrentEquivalence, // <|>
    PredictiveEquivalence, // </>
    RetrospectiveEquivalence, // <\>
    ParallelEvents,   // &|
    SequentialEvents, // &/
    List,             // #
    Op,               // ^
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Term {
    Atom(String),
    Var(VarType, String),
    Compound(Operator, Vec<Term>),
}

impl Term {
    pub fn atom_from_str(s: &str) -> Self {
        Term::Atom(s.to_string())
    }

    pub fn var_from_str(type_: VarType, s: &str) -> Self {
        Term::Var(type_, s.to_string())
    }
    
    pub fn to_display_string(&self) -> String {
        match self {
            Term::Atom(s) => s.clone(),
            Term::Var(t, s) => format!("{:?}:{}", t, s),
            Term::Compound(op, args) => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_display_string()).collect();
                format!("({:?} {:?})", op, args_str)
            }
        }
    }
}
