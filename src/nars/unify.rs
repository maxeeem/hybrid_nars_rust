use std::collections::HashMap;
use super::term::Term;

pub type Bindings = HashMap<Term, Term>;

pub fn unify(x: &Term, y: &Term) -> Option<Bindings> {
    unify_with_bindings(x, y, HashMap::new())
}

pub fn unify_with_bindings(x: &Term, y: &Term, bindings: Bindings) -> Option<Bindings> {
    unify_internal(x, y, bindings)
}

fn unify_internal(x: &Term, y: &Term, bindings: Bindings) -> Option<Bindings> {
    // Check if x or y are variables
    if let Term::Var(_, _) = x {
        return unify_var(x, y, bindings);
    }
    if let Term::Var(_, _) = y {
        return unify_var(y, x, bindings);
    }

    match (x, y) {
        (Term::Compound(op1, args1), Term::Compound(op2, args2)) => {
            if op1 != op2 || args1.len() != args2.len() {
                return None;
            }
            let mut current_bindings = bindings;
            for (arg1, arg2) in args1.iter().zip(args2.iter()) {
                if let Some(new_bindings) = unify_internal(arg1, arg2, current_bindings) {
                    current_bindings = new_bindings;
                } else {
                    return None;
                }
            }
            Some(current_bindings)
        }
        (Term::Atom(h1), Term::Atom(h2)) => {
            if h1 == h2 {
                Some(bindings)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn unify_var(var: &Term, x: &Term, mut bindings: Bindings) -> Option<Bindings> {
    if let Some(val) = bindings.get(var) {
        // Need to clone val because bindings is moved into unify_internal
        let val_clone = val.clone(); 
        return unify_internal(&val_clone, x, bindings);
    }
    if let Term::Var(_, _) = x {
        if let Some(val) = bindings.get(x) {
            let val_clone = val.clone();
            return unify_internal(var, &val_clone, bindings);
        }
    }
    if occurs_in(var, x, &bindings) {
        return None;
    }
    
    bindings.insert(var.clone(), x.clone());
    Some(bindings)
}

fn occurs_in(var: &Term, x: &Term, bindings: &Bindings) -> bool {
    if var == x {
        return true;
    }
    if let Term::Var(_, _) = x {
        if let Some(val) = bindings.get(x) {
            return occurs_in(var, val, bindings);
        }
    }
    if let Term::Compound(_, args) = x {
        for arg in args {
            if occurs_in(var, arg, bindings) {
                return true;
            }
        }
    }
    false
}
