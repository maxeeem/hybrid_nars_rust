use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarType {
    Independent, // $
    Dependent,   // #
    Query,       // ?
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    ParallelEvents,   // &|
    SequentialEvents, // &/
    List,             // #
    Op,               // ^
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term {
    Atom(u64),
    Var(VarType, u64),
    Compound(Operator, Vec<Term>),
}

impl Term {
    pub fn atom_from_str(s: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        Term::Atom(hasher.finish())
    }

    pub fn var_from_str(type_: VarType, s: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        Term::Var(type_, hasher.finish())
    }
}
