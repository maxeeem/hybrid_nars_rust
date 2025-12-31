use std::hash::{Hash, Hasher};
use serde::{Serialize, Deserialize};

// Deterministic hash function (FNV-1a)
fn deterministic_hash(s: &str) -> u64 {
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
    Atom(u64),
    Var(VarType, u64),
    Compound(Operator, Vec<Term>),
}

impl Term {
    pub fn atom_from_str(s: &str) -> Self {
        Term::Atom(deterministic_hash(s))
    }

    pub fn var_from_str(type_: VarType, s: &str) -> Self {
        Term::Var(type_, deterministic_hash(s))
    }
}
