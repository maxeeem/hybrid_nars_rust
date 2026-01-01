use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    character::complete::{char, multispace0},
    combinator::{map, value},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
    Parser,
};
use super::rules::{InferenceRule, TruthFunction};
use super::term::{Term, Operator, VarType};
use super::truth;

// --- Parsing Logic (Adapted from rule_loader.rs) ---

#[derive(Debug, Clone, PartialEq)]
enum Sexp {
    Atom(String),
    List(Vec<Sexp>),
}

fn is_symbol_char(c: char) -> bool {
    !c.is_whitespace() && c != '(' && c != ')' && c != ';'
}

fn parse_atom(input: &str) -> IResult<&str, Sexp> {
    map(take_while1(is_symbol_char), |s: &str| Sexp::Atom(s.to_string())).parse(input)
}

fn parse_comment(input: &str) -> IResult<&str, ()> {
    value(
        (),
        pair(char(';'), take_while(|c| c != '\n' && c != '\r')),
    ).parse(input)
}

fn parse_sexp(input: &str) -> IResult<&str, Sexp> {
    let (input, _) = multispace0(input)?;
    let (input, _) = many0((parse_comment, multispace0)).parse(input)?;
    
    alt((
        parse_atom,
        map(
            delimited(
                char('('),
                many0(parse_sexp),
                preceded(multispace0, char(')')),
            ),
            Sexp::List,
        ),
    )).parse(input)
}

fn parse_term_from_sexp(sexp: &Sexp) -> Option<Term> {
    match sexp {
        Sexp::Atom(s) => {
            if s.starts_with(':') {
                Some(Term::var_from_str(VarType::Independent, &s[1..]))
            } else if s.starts_with("$") {
                Some(Term::var_from_str(VarType::Independent, &s[1..]))
            } else if s.starts_with("#") {
                Some(Term::var_from_str(VarType::Dependent, &s[1..]))
            } else if s.starts_with("?") {
                Some(Term::var_from_str(VarType::Query, &s[1..]))
            } else {
                Some(Term::atom_from_str(s))
            }
        }
        Sexp::List(list) => {
            if list.is_empty() {
                return None;
            }
            
            // Handle single element list (parens around a term)
            if list.len() == 1 {
                return parse_term_from_sexp(&list[0]);
            }

            // Check for infix notation like (:S --> :P)
            if list.len() == 3 {
                if let Sexp::Atom(op_str) = &list[1] {
                    let op = match op_str.as_str() {
                        "-->" => Some(Operator::Inheritance),
                        "==>" => Some(Operator::Implication),
                        "<->" => Some(Operator::Similarity),
                        "<=>" => Some(Operator::Equivalence),
                        _ => None,
                    };
                    
                    if let Some(operator) = op {
                        let subject = parse_term_from_sexp(&list[0])?;
                        let predicate = parse_term_from_sexp(&list[2])?;
                        return Some(Term::Compound(operator, vec![subject, predicate]));
                    }
                }
            }

            // Prefix notation or other compounds
            if let Sexp::Atom(op_str) = &list[0] {
                let op = match op_str.as_str() {
                    "&" => Operator::IntIntersection,
                    "|" => Operator::ExtIntersection,
                    "+" => Operator::Union,
                    "-" => Operator::Difference,
                    "~" => Operator::DifferenceInt,
                    "--" => Operator::Negation,
                    _ => return None, // Unknown operator
                };
                
                let mut args = Vec::new();
                for arg_sexp in &list[1..] {
                    args.push(parse_term_from_sexp(arg_sexp)?);
                }
                return Some(Term::Compound(op, args));
            }
            
            None
        }
    }
}

fn parse_term_str(input: &str) -> Term {
    let (_, sexp) = parse_sexp(input).expect(&format!("Failed to parse term string: {}", input));
    parse_term_from_sexp(&sexp).expect(&format!("Failed to convert Sexp to Term: {}", input))
}

fn get_truth_fn(name: &str) -> TruthFunction {
    match name {
        "deduction" => TruthFunction::Double(truth::deduction),
        "abduction" => TruthFunction::Double(truth::abduction),
        "induction" => TruthFunction::Double(truth::induction),
        "exemplification" => TruthFunction::Double(truth::exemplification),
        "intersection" => TruthFunction::Double(truth::intersection),
        "comparison" => TruthFunction::Double(truth::comparison),
        "analogy" => TruthFunction::Double(truth::analogy),
        "resemblance" => TruthFunction::Double(truth::resemblance),
        "conversion" => TruthFunction::Single(truth::conversion),
        "contraposition" => TruthFunction::Single(truth::contraposition),
        "negation" => TruthFunction::Single(truth::negation),
        "union" => TruthFunction::Double(truth::union),
        "difference" => TruthFunction::Double(truth::difference),
        "decomposition" => TruthFunction::Double(truth::decompose_ppp),
        "reduce_disjunction" => TruthFunction::Double(truth::reduce_disjunction),
        "structural_deduction" => TruthFunction::Single(truth::structural_deduction),
        _ => panic!("Unknown truth function: {}", name),
    }
}

// --- Macro and Rules ---

macro_rules! rule {
    ($p1:literal !- $conc:literal $truth:literal) => {
        InferenceRule {
            name: $truth.to_string(),
            premises: vec![parse_term_str($p1)],
            conclusion: parse_term_str($conc),
            truth_fn: get_truth_fn($truth),
        }
    };
    ($p1:literal $p2:literal !- $conc:literal $truth:literal) => {
        InferenceRule {
            name: $truth.to_string(),
            premises: vec![parse_term_str($p1), parse_term_str($p2)],
            conclusion: parse_term_str($conc),
            truth_fn: get_truth_fn($truth),
        }
    };
}

pub fn get_all_rules() -> Vec<InferenceRule> {
    let mut rules = Vec::new();

    // --- IMMEDIATE INFERENCE ---
    rules.push(rule!("(-- :M)"                  !- "(:M)"                    "negation"));
    rules.push(rule!("(:S --> :P)"              !- "(:P --> :S)"             "conversion"));
    rules.push(rule!("(:S ==> :P)"              !- "(:P ==> :S)"             "conversion"));
    rules.push(rule!("(:S ==> :P)"              !- "((-- :P) ==> (-- :S))"   "contraposition"));

    // --- SYLLOGISMS (NAL-1) ---
    rules.push(rule!("(:M --> :P)" "(:S --> :M)"  !- "(:S --> :P)"             "deduction"));
    rules.push(rule!("(:P --> :M)" "(:S --> :M)"  !- "(:S --> :P)"             "abduction"));
    rules.push(rule!("(:M --> :P)" "(:M --> :S)"  !- "(:S --> :P)"             "induction"));
    rules.push(rule!("(:P --> :M)" "(:M --> :S)"  !- "(:S --> :P)"             "exemplification"));

    // --- SYLLOGISMS (NAL-2) ---
    rules.push(rule!("(:S --> :P)" "(:P --> :S)"  !- "(:P <-> :S)"             "intersection"));
    rules.push(rule!("(:M --> :P)" "(:S <-> :M)"  !- "(:S --> :P)"             "analogy"));
    rules.push(rule!("(:P --> :M)" "(:S <-> :M)"  !- "(:P --> :S)"             "analogy"));
    rules.push(rule!("(:M <-> :P)" "(:S <-> :M)"  !- "(:P <-> :S)"             "resemblance"));

    // --- HIGHER ORDER (NAL-5) ---
    rules.push(rule!("(:M ==> :P)" "(:S ==> :M)"  !- "(:S ==> :P)"             "deduction"));
    rules.push(rule!("(:P ==> :M)" "(:S ==> :M)"  !- "(:S ==> :P)"             "abduction"));
    rules.push(rule!("(:M ==> :P)" "(:M ==> :S)"  !- "(:S ==> :P)"             "induction"));
    rules.push(rule!("(:S ==> :P)" "(:P ==> :S)"  !- "(:S <=> :P)"             "intersection"));
    rules.push(rule!("(:M ==> :P)" "(:S <=> :M)"  !- "(:S ==> :P)"             "analogy"));
    rules.push(rule!("(:M <=> :P)" "(:S <=> :M)"  !- "(:S <=> :P)"             "resemblance"));

    // --- VARIABLES (NAL-6) ---
    rules.push(rule!("(:S --> :M)" "(:P --> :M)"  !- "((:P --> $X) ==> (:S --> $X))" "abduction"));
    rules.push(rule!("(:S --> :M)" "(:P --> :M)"  !- "((:S --> $X) ==> (:P --> $X))" "induction"));
    rules.push(rule!("(:M --> :S)" "(:M --> :P)"  !- "(($X --> :S) ==> ($X --> :P))" "induction"));
    rules.push(rule!("(:M --> :S)" "(:M --> :P)"  !- "(($X --> :P) ==> ($X --> :S))" "abduction"));

    // --- SETS & COMPOSITION (NAL-3) ---
    // Intersection (&)
    rules.push(rule!("(:P --> :M) (:S --> :M)" !- "((& :S :P) --> :M)" "intersection"));
    rules.push(rule!("(:M --> :P) (:M --> :S)" !- "(:M --> (& :P :S))" "intersection"));
    
    // Union (+) - mapped to 'union' truth fn
    rules.push(rule!("(:P --> :M) (:S --> :M)" !- "((+ :S :P) --> :M)" "union"));
    rules.push(rule!("(:M --> :P) (:M --> :S)" !- "(:M --> (+ :P :S))" "union"));
    
    // Difference (-) and (~)
    rules.push(rule!("(:P --> :M) (:S --> :M)" !- "((~ :P :S) --> :M)" "difference"));
    rules.push(rule!("(:M --> :P) (:M --> :S)" !- "(:M --> (- :P :S))" "difference"));

    // --- DECOMPOSITION (NAL-3) ---
    // Simplification for Sets
    rules.push(rule!("(:S --> :M) ((& :S :P) --> :M)" !- "(:P --> :M)" "decomposition"));
    rules.push(rule!("(:M --> :S) (:M --> (& :S :P))" !- "(:M --> :P)" "decomposition"));

    // Disjunction Decomposition
    rules.push(rule!("(:S --> (| :P :M)) (:S --> :M)" !- "(:S --> :P)" "reduce_disjunction"));
    rules.push(rule!("(:S --> (| :M :P)) (:S --> :M)" !- "(:S --> :P)" "reduce_disjunction"));

    // Structural Decomposition (Single Premise)
    rules.push(rule!("((| :S :P) --> :M)" !- "(:S --> :M)" "structural_deduction"));
    rules.push(rule!("((| :P :S) --> :M)" !- "(:S --> :M)" "structural_deduction"));
    rules.push(rule!("(:M --> (& :S :P))" !- "(:M --> :S)" "structural_deduction"));
    rules.push(rule!("(:M --> (& :P :S))" !- "(:M --> :S)" "structural_deduction"));
    rules.push(rule!("(:M --> (| :S :P))" !- "(:M --> :S)" "structural_deduction"));
    rules.push(rule!("(:M --> (| :P :S))" !- "(:M --> :S)" "structural_deduction"));

    rules
}
