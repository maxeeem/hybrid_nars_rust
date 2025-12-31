use std::fs;
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

fn parse_file(input: &str) -> IResult<&str, Vec<Sexp>> {
    many0(parse_sexp).parse(input)
}

fn parse_term(sexp: &Sexp) -> Option<Term> {
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
                        let subject = parse_term(&list[0])?;
                        let predicate = parse_term(&list[2])?;
                        return Some(Term::Compound(operator, vec![subject, predicate]));
                    }
                }
            }

            // Prefix notation or other compounds
            if let Sexp::Atom(op_str) = &list[0] {
                let op = match op_str.as_str() {
                    "&" => Operator::IntIntersection,
                    "|" => Operator::ExtIntersection,
                    "-" => Operator::Difference,
                    "~" => Operator::Difference,
                    "--" => Operator::Negation,
                    "&&" => Operator::Conjunction,
                    "||" => Operator::Disjunction,
                    "*" => Operator::Product,
                    "/" => Operator::ExtImage,
                    "\\" => Operator::IntImage,
                    "{}" => Operator::ExtSet,
                    "[]" => Operator::IntSet,
                    _ => Operator::Other(op_str.clone()),
                };
                
                let mut args = Vec::new();
                for item in &list[1..] {
                    args.push(parse_term(item)?);
                }
                return Some(Term::Compound(op, args));
            }
            
            None
        }
    }
}

fn get_truth_fn(name: &str) -> Option<TruthFunction> {
    match name {
        ":t/deduction" => Some(TruthFunction::Double(truth::deduction)),
        ":t/abduction" => Some(TruthFunction::Double(truth::abduction)),
        ":t/induction" => Some(TruthFunction::Double(truth::induction)),
        ":t/exemplification" => Some(TruthFunction::Double(truth::exemplification)),
        ":t/comparison" => Some(TruthFunction::Double(truth::comparison)),
        ":t/analogy" => Some(TruthFunction::Double(truth::analogy)),
        ":t/resemblance" => Some(TruthFunction::Double(truth::resemblance)),
        ":t/intersection" => Some(TruthFunction::Double(truth::intersection)),
        ":t/union" => Some(TruthFunction::Double(truth::union)),
        ":t/difference" => Some(TruthFunction::Double(truth::difference)),
        ":t/conversion" => Some(TruthFunction::Single(truth::conversion)),
        ":t/contraposition" => Some(TruthFunction::Single(truth::contraposition)),
        ":t/negation" => Some(TruthFunction::Single(nal_negation)),
        _ => None,
    }
}

fn nal_negation(v: truth::TruthValue) -> truth::TruthValue {
    truth::TruthValue::new(truth::nal_not(v.frequency), v.confidence)
}

pub fn load_rules(path: &str) -> Vec<InferenceRule> {
    let content = fs::read_to_string(path).expect("Failed to read rules file");
    let (_, sexps) = parse_file(&content).expect("Failed to parse rules file");
    
    let mut rules = Vec::new();

    for top_level in sexps {
        if let Sexp::List(items) = top_level {
            if items.is_empty() { continue; }
            
            // Iterate over the rules inside the definition
            // The format is (define-mediate-rules *name* rule1 rule2 ...)
            for rule_sexp in &items[2..] {
                if let Sexp::List(rule_parts) = rule_sexp {
                    // Find "!-"
                    if let Some(split_idx) = rule_parts.iter().position(|x| matches!(x, Sexp::Atom(s) if s == "!-")) {
                        let premises_sexps = &rule_parts[0..split_idx];
                        let conclusions_sexps = &rule_parts[split_idx+1]; // This is a list of conclusions
                        
                        // Parse premises
                        let mut premises = Vec::new();
                        for p in premises_sexps {
                            if let Some(term) = parse_term(p) {
                                premises.push(term);
                            }
                        }
                        
                        // Parse conclusions
                        if let Sexp::List(conclusions_list) = conclusions_sexps {
                            for concl_def in conclusions_list {
                                if let Sexp::List(parts) = concl_def {
                                    if parts.len() >= 2 {
                                        let term_sexp = &parts[0];
                                        let truth_info = &parts[1];
                                        
                                        if let Some(term) = parse_term(term_sexp) {
                                            // Extract truth function
                                            let mut truth_fn = None;
                                            if let Sexp::List(tf_parts) = truth_info {
                                                for tf_part in tf_parts {
                                                    if let Sexp::Atom(s) = tf_part {
                                                        if let Some(tf) = get_truth_fn(s) {
                                                            truth_fn = Some(tf);
                                                            break;
                                                        }
                                                        // Special case for negation which might be named differently
                                                        if s == ":t/negation" {
                                                            truth_fn = Some(TruthFunction::Single(nal_negation));
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            if let Some(tf) = truth_fn {
                                                rules.push(InferenceRule {
                                                    premises: premises.clone(),
                                                    conclusion: term,
                                                    truth_fn: tf,
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    rules
}


/*
fn parse_sexpr(input: &str) -> Vec<SExpr> {
    let mut tokens = tokenize(input);
    let mut exprs = Vec::new();
    while !tokens.is_empty() {
        if let Some(expr) = parse_one(&mut tokens) {
            exprs.push(expr);
        } else {
            break;
        }
    }
    exprs
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            '(' | ')' => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(c.to_string());
                chars.next();
            }
            ';' => {
                // Comment, skip until newline
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                while let Some(&nc) = chars.peek() {
                    if nc == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            c if c.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                chars.next();
            }
            _ => {
                current.push(c);
                chars.next();
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn parse_one(tokens: &mut Vec<String>) -> Option<SExpr> {
    if tokens.is_empty() {
        return None;
    }
    let token = tokens.remove(0);
    if token == "(" {
        let mut list = Vec::new();
        while !tokens.is_empty() && tokens[0] != ")" {
            if let Some(expr) = parse_one(tokens) {
                list.push(expr);
            }
        }
        if !tokens.is_empty() && tokens[0] == ")" {
            tokens.remove(0);
        }
        Some(SExpr::List(list))
    } else if token == ")" {
        // Should not happen if balanced
        None
    } else {
        Some(SExpr::Atom(token))
    }
}

fn parse_term(expr: &SExpr) -> Option<Term> {
    match expr {
        SExpr::Atom(s) => {
            if s.starts_with(':') {
                // Variable
                let name = &s[1..];
                Some(Term::var_from_str(VarType::Independent, name))
            } else {
                // Atom or Operator?
                // In the rules, atoms are usually variables like :S, :P
                // But sometimes we might have constants.
                // For now assume everything else is an atom if it's not a keyword
                Some(Term::atom_from_str(s))
            }
        }
        SExpr::List(list) => {
            // (Subject Operator Predicate) or (Operator Arg1 Arg2 ...)
            // The rules use infix: (:S --> :P)
            if list.len() == 3 {
                let op_str = match &list[1] {
                    SExpr::Atom(s) => s,
                    _ => return None,
                };
                
                let op = match op_str.as_str() {
                    "-->" => Operator::Inheritance,
                    "==>" => Operator::Implication,
                    "<=>" => Operator::Equivalence,
                    "<->" => Operator::Similarity,
                    _ => return None, // Unknown operator
                };

                let subject = parse_term(&list[0])?;
                let predicate = parse_term(&list[2])?;

                Some(Term::Compound(op, vec![subject, predicate]))
            } else if list.len() == 2 {
                // Prefix operator like (-- :P)
                let op_str = match &list[0] {
                    SExpr::Atom(s) => s,
                    _ => return None,
                };
                
                let op = match op_str.as_str() {
                    "--" => Operator::Negation,
                    _ => return None,
                };
                
                let arg = parse_term(&list[1])?;
                Some(Term::Compound(op, vec![arg]))
            } else {
                None
            }
        }
    }
}

fn get_truth_fn(name: &str) -> TruthFunction {
    match name {
        ":t/deduction" => TruthFunction::Double(truth::deduction),
        ":t/abduction" => TruthFunction::Double(truth::abduction),
        ":t/induction" => TruthFunction::Double(truth::induction),
        ":t/exemplification" => TruthFunction::Double(truth::exemplification),
        ":t/intersection" => TruthFunction::Double(truth::intersection),
        ":t/resemblance" => TruthFunction::Double(truth::resemblance),
        ":t/analogy" => TruthFunction::Double(truth::analogy),
        ":t/comparison" => TruthFunction::Double(truth::comparison),
        ":t/conversion" => TruthFunction::Single(truth::conversion),
        ":t/contraposition" => TruthFunction::Single(truth::contraposition),
        _ => TruthFunction::Double(truth::deduction), // Default or panic?
    }
}

pub fn load_rules(path: &str) -> Vec<InferenceRule> {
    let content = fs::read_to_string(path).expect("Failed to read rules file");
    let exprs = parse_sexpr(&content);
    let mut rules = Vec::new();

    for expr in exprs {
        if let SExpr::List(list) = expr {
            if list.is_empty() { continue; }
            // (define-mediate-rules *name* rule1 rule2 ...)
            if let SExpr::Atom(s) = &list[0] {
                if s == "define-mediate-rules" || s == "define-immediate-rules" {
                    // Iterate over rules starting from index 2
                    for rule_expr in list.iter().skip(2) {
                        if let SExpr::List(rule_parts) = rule_expr {
                            // Rule structure: (Premise1 Premise2 ... !- (ConclusionBlock ...))
                            // We need to find "!-"
                            let mut premises = Vec::new();

                            for part in rule_parts {
                                if let SExpr::Atom(s) = part {
                                    if s == "!-" {
                                        continue;
                                    }
                                    // Keys like :substitutions
                                    if s.starts_with(':') {
                                        break; // End of premises/conclusion part
                                    }
                                }
                                
                                if let SExpr::List(l) = part {
                                    // Check if it's the conclusion block
                                    // Conclusion block looks like: ((:S --> :P) (:t/deduction ...))
                                    // Or premises: (:M --> :P)
                                    
                                    // Heuristic: if we already passed !-, this is the conclusion block
                                    // But my loop structure is simple.
                                    // Let's split by !- index.
                                }
                            }

                            // Better approach: split the list by "!-" atom
                            let split_idx = rule_parts.iter().position(|x| matches!(x, SExpr::Atom(s) if s == "!-"));
                            
                            if let Some(idx) = split_idx {
                                // Premises are before idx
                                for i in 0..idx {
                                    if let Some(term) = parse_term(&rule_parts[i]) {
                                        premises.push(term);
                                    }
                                }

                                // Conclusion block is at idx + 1
                                if idx + 1 < rule_parts.len() {
                                    if let SExpr::List(concl_list) = &rule_parts[idx + 1] {
                                        // ((:S --> :P) (:t/deduction ...))
                                        // Sometimes it's a list of conclusions?
                                        // The example shows: (((:S --> :P) (:t/deduction :d/strong)))
                                        // Wait, look at the file:
                                        // ((:M --> :P) (:S --> :M) !- (((:S --> :P) (:t/deduction :d/strong)))
                                        // So the element after !- is a List of (Conclusion, TruthFn) pairs.
                                        
                                        for concl_pair in concl_list {
                                            if let SExpr::List(pair) = concl_pair {
                                                if pair.len() >= 2 {
                                                    let conclusion = parse_term(&pair[0]);
                                                    let truth_fn_name = if let SExpr::List(tf) = &pair[1] {
                                                        // (:t/deduction :d/strong)
                                                        if let SExpr::Atom(n) = &tf[0] {
                                                            Some(n.clone())
                                                        } else { None }
                                                    } else { None };

                                                    if let (Some(c), Some(tf_name)) = (conclusion, truth_fn_name) {
                                                        rules.push(InferenceRule {
                                                            premises: premises.clone(),
                                                            conclusion: c,
                                                            truth_fn: get_truth_fn(&tf_name),
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    rules
}
*/
