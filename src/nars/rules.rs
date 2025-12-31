use super::term::{Term, Operator, VarType};
use super::truth::{self, TruthValue};

#[derive(Clone, Copy)]
pub enum TruthFunction {
    Single(fn(TruthValue) -> TruthValue),
    Double(fn(TruthValue, TruthValue) -> TruthValue),
}

pub struct InferenceRule {
    pub premises: Vec<Term>,
    pub conclusion: Term,
    pub truth_fn: TruthFunction,
}

pub fn load_default_rules() -> Vec<InferenceRule> {
    let mut rules = Vec::new();

    // Helper to create variables
    let var_m = Term::var_from_str(VarType::Independent, "M");
    let var_p = Term::var_from_str(VarType::Independent, "P");
    let var_s = Term::var_from_str(VarType::Independent, "S");

    // Deduction: ((:M --> :P), (:S --> :M)) |- (:S --> :P)
    // Premise 1: <$M --> $P>
    let ded_p1 = Term::Compound(Operator::Inheritance, vec![var_m.clone(), var_p.clone()]);
    // Premise 2: <$S --> $M>
    let ded_p2 = Term::Compound(Operator::Inheritance, vec![var_s.clone(), var_m.clone()]);
    // Conclusion: <$S --> $P>
    let ded_concl = Term::Compound(Operator::Inheritance, vec![var_s.clone(), var_p.clone()]);

    rules.push(InferenceRule {
        premises: vec![ded_p1, ded_p2],
        conclusion: ded_concl,
        truth_fn: TruthFunction::Double(truth::deduction),
    });

    // Abduction: ((:P --> :M), (:S --> :M)) |- (:S --> :P)
    // Premise 1: <$P --> $M>
    let abd_p1 = Term::Compound(Operator::Inheritance, vec![var_p.clone(), var_m.clone()]);
    // Premise 2: <$S --> $M>
    let abd_p2 = Term::Compound(Operator::Inheritance, vec![var_s.clone(), var_m.clone()]);
    // Conclusion: <$S --> $P>
    let abd_concl = Term::Compound(Operator::Inheritance, vec![var_s.clone(), var_p.clone()]);

    rules.push(InferenceRule {
        premises: vec![abd_p1, abd_p2],
        conclusion: abd_concl,
        truth_fn: TruthFunction::Double(truth::abduction),
    });

    // Induction: ((:M --> :P), (:M --> :S)) |- (:S --> :P)
    // Premise 1: <$M --> $P>
    let ind_p1 = Term::Compound(Operator::Inheritance, vec![var_m.clone(), var_p.clone()]);
    // Premise 2: <$M --> $S>
    let ind_p2 = Term::Compound(Operator::Inheritance, vec![var_m.clone(), var_s.clone()]);
    // Conclusion: <$S --> $P>
    let ind_concl = Term::Compound(Operator::Inheritance, vec![var_s.clone(), var_p.clone()]);

    rules.push(InferenceRule {
        premises: vec![ind_p1, ind_p2],
        conclusion: ind_concl,
        truth_fn: TruthFunction::Double(truth::induction),
    });

    rules
}
