use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TruthValue {
    pub frequency: f32,
    pub confidence: f32,
}

impl TruthValue {
    pub fn new(frequency: f32, confidence: f32) -> Self {
        Self { frequency, confidence }
    }
}

// Helper functions
pub fn nal_and(values: &[f32]) -> f32 {
    values.iter().product()
}

pub fn nal_or(values: &[f32]) -> f32 {
    1.0 - values.iter().map(|&i| 1.0 - i).product::<f32>()
}

pub fn nal_not(x: f32) -> f32 {
    1.0 - x
}

fn safe_div(x: f32, y: f32) -> f32 {
    if y == 0.0 {
        0.0
    } else {
        x / y
    }
}

// Truth Functions

pub fn revision(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    let f = safe_div(
        nal_and(&[f1, c1, nal_not(c2)]) + nal_and(&[f2, c2, nal_not(c1)]),
        nal_and(&[c1, nal_not(c2)]) + nal_and(&[c2, nal_not(c1)])
    );
    
    let c = safe_div(
        nal_and(&[c1, nal_not(c2)]) + nal_and(&[c2, nal_not(c1)]),
        nal_and(&[c1, nal_not(c2)]) + nal_and(&[c2, nal_not(c1)]) + nal_and(&[nal_not(c1), nal_not(c2)])
    );

    TruthValue::new(f, c)
}

pub fn union(v1: TruthValue, v2: TruthValue) -> TruthValue {
    TruthValue::new(
        nal_or(&[v1.frequency, v2.frequency]),
        nal_and(&[v1.confidence, v2.confidence])
    )
}

pub fn difference(v1: TruthValue, v2: TruthValue) -> TruthValue {
    TruthValue::new(
        nal_and(&[v1.frequency, nal_not(v2.frequency)]),
        nal_and(&[v1.confidence, v2.confidence])
    )
}

pub fn intersection(v1: TruthValue, v2: TruthValue) -> TruthValue {
    TruthValue::new(
        nal_and(&[v1.frequency, v2.frequency]),
        nal_and(&[v1.confidence, v2.confidence])
    )
}

pub fn deduction(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    TruthValue::new(
        nal_and(&[f1, f2]),
        nal_and(&[f1, c1, f2, c2])
    )
}

pub fn contraposition(v: TruthValue) -> TruthValue {
    let f = v.frequency;
    let c = v.confidence;
    let k = 1.0;

    TruthValue::new(
        0.0,
        safe_div(nal_and(&[nal_not(f), c]), nal_and(&[nal_not(f), c]) + k)
    )
}

pub fn abduction(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let _f2 = v2.frequency; // f2 is used as result frequency
    let c2 = v2.confidence;
    let k = 1.0;

    TruthValue::new(
        v2.frequency,
        safe_div(nal_and(&[f1, c1, c2]), nal_and(&[f1, c1, c2]) + k)
    )
}

pub fn exemplification(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;
    let k = 1.0;

    TruthValue::new(
        1.0,
        safe_div(nal_and(&[f1, c1, f2, c2]), nal_and(&[f1, c1, f2, c2]) + k)
    )
}

pub fn induction(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency; // f1 is used as result frequency
    let _c1 = v1.confidence;
    let f2 = v2.frequency;
    let c1 = v1.confidence;
    let c2 = v2.confidence;
    let k = 1.0;

    TruthValue::new(
        f1,
        safe_div(nal_and(&[f2, c1, c2]), nal_and(&[f2, c1, c2]) + k)
    )
}

pub fn comparison(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;
    let k = 1.0;

    let f0 = nal_or(&[f1, f2]);
    let w = nal_and(&[f0, c1, c2]);
    
    let f = safe_div(nal_and(&[f1, f2, c1, c2]), w);
    let c = safe_div(w, w + k);

    TruthValue::new(f, c)
}

pub fn desire_weak(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;
    let k = 1.0;

    TruthValue::new(
        nal_and(&[f1, f2]),
        nal_and(&[c1, c2, f2, 1.0 / (1.0 + k)])
    )
}

pub fn temporal_induction(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;
    let k = 1.0;

    TruthValue::new(
        f1,
        safe_div(nal_and(&[f2, c1, c2]), nal_and(&[f2, c1, c2]) + k)
    )
}

pub fn resemblance(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    TruthValue::new(
        nal_and(&[f1, f2]),
        nal_and(&[nal_or(&[f1, f2]), c1, c2])
    )
}

pub fn analogy(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    TruthValue::new(
        nal_and(&[f1, f2]),
        nal_and(&[f2, c1, c2])
    )
}

pub fn decompose_nnn(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    let term = nal_and(&[nal_not(f1), nal_not(f2)]);
    TruthValue::new(
        nal_not(term),
        nal_and(&[term, c1, c2])
    )
}

pub fn decompose_npp(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    let term = nal_and(&[nal_not(f1), f2]);
    TruthValue::new(
        term,
        nal_and(&[term, c1, c2])
    )
}

pub fn decompose_ppp(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    let term = nal_and(&[f1, f2]);
    TruthValue::new(
        term,
        nal_and(&[term, c1, c2])
    )
}

pub fn decompose_pnn(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    let term = nal_and(&[f1, nal_not(f2)]);
    TruthValue::new(
        nal_not(term),
        nal_and(&[nal_not(term), c1, c2])
    )
}

pub fn decompose_pnp(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    let term = nal_and(&[f1, nal_not(f2)]);
    TruthValue::new(
        term,
        nal_and(&[term, c1, c2])
    )
}

pub fn desire_strong(v1: TruthValue, v2: TruthValue) -> TruthValue {
    let f1 = v1.frequency;
    let c1 = v1.confidence;
    let f2 = v2.frequency;
    let c2 = v2.confidence;

    TruthValue::new(
        nal_and(&[f1, f2]),
        nal_and(&[f2, c1, c2])
    )
}

pub fn combine(v1: TruthValue, v2: TruthValue) -> TruthValue {
    TruthValue::new(
        nal_and(&[v1.frequency, v2.frequency]),
        nal_and(&[v1.confidence, v2.confidence])
    )
}

pub fn assumption_of_failure() -> TruthValue {
    TruthValue::new(0.0, 0.15)
}

pub fn identity(v: TruthValue) -> TruthValue {
    v
}

pub fn negation(v: TruthValue) -> TruthValue {
    TruthValue::new(nal_not(v.frequency), v.confidence)
}

pub fn structural_deduction(v: TruthValue) -> TruthValue {
    let f = v.frequency;
    let c = v.confidence;
    TruthValue::new(f, nal_and(&[f, c, c]))
}

pub fn desire_structural_strong(v: TruthValue) -> TruthValue {
    let f = v.frequency;
    let c = v.confidence;
    TruthValue::new(f, nal_and(&[f, c, c]))
}

pub fn conversion(v: TruthValue) -> TruthValue {
    let f = v.frequency;
    let c = v.confidence;
    let k = 1.0;
    TruthValue::new(
        f,
        safe_div(nal_and(&[f, c]), nal_and(&[f, c]) + k)
    )
}
