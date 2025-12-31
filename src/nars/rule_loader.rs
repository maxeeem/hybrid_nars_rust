use std::fs;
use super::term::{Term, Operator, VarType};
use super::rules::{InferenceRule, TruthFunction};
use super::truth::{self, TruthValue};

#[derive(Debug, Clone)]
enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

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
                                        // The example shows: (((:S --> :P) (:t/deduction ...)))
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
