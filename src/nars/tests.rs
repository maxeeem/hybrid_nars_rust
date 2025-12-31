#[cfg(test)]
mod tests {
    use crate::nars::term::{Term, Operator, VarType};
    use crate::nars::truth::{self, TruthValue};
    use crate::nars::unify::unify;

    #[test]
    fn test_math_deduction() {
        let v1 = TruthValue::new(1.0, 0.9);
        let v2 = TruthValue::new(1.0, 0.9);
        let result = truth::deduction(v1, v2);
        
        // Expected: f=1.0, c=0.81
        // Formula: c = f1 * c1 * f2 * c2 = 1.0 * 0.9 * 1.0 * 0.9 = 0.81
        // f = f1 * f2 = 1.0 * 1.0 = 1.0
        
        let epsilon = 1e-6;
        assert!((result.frequency - 1.0).abs() < epsilon, "Frequency mismatch: expected 1.0, got {}", result.frequency);
        assert!((result.confidence - 0.81).abs() < epsilon, "Confidence mismatch: expected 0.81, got {}", result.confidence);
    }

    #[test]
    fn test_unification() {
        // Helper to create atoms with fixed IDs for determinism
        let atom = |id| Term::Atom(id);
        let var = |id| Term::Var(VarType::Independent, id);
        
        // IDs
        let id_x = 100;
        let id_duck = 1;
        let id_bird = 2;
        let id_swimmer = 3;
        let id_fish = 4;

        // Terms
        let x = var(id_x);
        let duck = atom(id_duck);
        let bird = atom(id_bird);
        let swimmer = atom(id_swimmer);
        let fish = atom(id_fish);

        // Rule: (&&, <$x --> bird>, <$x --> swimmer>)
        let rule = Term::Compound(Operator::Conjunction, vec![
            Term::Compound(Operator::Inheritance, vec![x.clone(), bird.clone()]),
            Term::Compound(Operator::Inheritance, vec![x.clone(), swimmer.clone()]),
        ]);

        // Fact: (&&, <duck --> bird>, <duck --> swimmer>)
        let fact = Term::Compound(Operator::Conjunction, vec![
            Term::Compound(Operator::Inheritance, vec![duck.clone(), bird.clone()]),
            Term::Compound(Operator::Inheritance, vec![duck.clone(), swimmer.clone()]),
        ]);

        // Execute Unification
        let bindings = unify(&rule, &fact);
        
        // Verify Success
        assert!(bindings.is_some(), "Unification failed");
        let bindings = bindings.unwrap();
        
        // Verify Binding { $x: duck }
        assert_eq!(bindings.get(&x), Some(&duck), "Binding mismatch for $x");
        assert_eq!(bindings.len(), 1, "Too many bindings");

        // Negative Test
        // Fact 2: (&&, <duck --> bird>, <fish --> swimmer>)
        // Here the first part <duck --> bird> matches <$x --> bird> binding $x to duck.
        // The second part <fish --> swimmer> tries to match <$x --> swimmer>.
        // Since $x is bound to duck, it checks <duck --> swimmer> vs <fish --> swimmer>.
        // duck != fish, so it should fail.
        let fact_neg = Term::Compound(Operator::Conjunction, vec![
            Term::Compound(Operator::Inheritance, vec![duck.clone(), bird.clone()]),
            Term::Compound(Operator::Inheritance, vec![fish.clone(), swimmer.clone()]),
        ]);

        let bindings_neg = unify(&rule, &fact_neg);
        assert!(bindings_neg.is_none(), "Unification should have failed for negative test");
    }
}
