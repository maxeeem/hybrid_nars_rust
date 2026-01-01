use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use super::bag::Bag;
use super::term::{Term, Operator, deterministic_hash};
use super::truth::TruthValue;
use super::sentence::{Sentence, Stamp};
use serde::{Serialize, Deserialize};
use serde_big_array::BigArray;

const HV_DIM_U64: usize = 157; // 157 * 64 = 10048 bits
const HV_DIM_BITS: usize = HV_DIM_U64 * 64;

pub struct ProjectionMatrix {
    weights: Vec<Vec<f32>>, // [bit_idx][input_dim]
}

impl ProjectionMatrix {
    pub fn new(input_dim: usize) -> Self {
        let mut weights = Vec::with_capacity(HV_DIM_BITS);
        for bit_idx in 0..HV_DIM_BITS {
            let mut rng = StdRng::seed_from_u64(bit_idx as u64);
            let mut row = Vec::with_capacity(input_dim);
            for _ in 0..input_dim {
                row.push(rng.random_range(-1.0..1.0));
            }
            weights.push(row);
        }
        Self { weights }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hypervector {
    #[serde(with = "BigArray")]
    pub bits: [u64; HV_DIM_U64],
}

impl Hypervector {
    /// Returns a vector of all zeros (empty accumulator).
    pub fn empty() -> Self {
        Self {
            bits: [0; HV_DIM_U64],
        }
    }

    /// Returns a random hypervector (for testing or initialization).
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let mut bits = [0; HV_DIM_U64];
        for i in 0..HV_DIM_U64 {
            bits[i] = rng.random();
        }
        Self { bits }
    }

    /// Bitwise XOR (Binding).
    pub fn bind(&self, other: &Hypervector) -> Hypervector {
        let mut result = [0; HV_DIM_U64];
        for i in 0..HV_DIM_U64 {
            result[i] = self.bits[i] ^ other.bits[i];
        }
        Self { bits: result }
    }

    /// The Majority Function (Bundling).
    pub fn bundle(inputs: &[Hypervector]) -> Hypervector {
        if inputs.is_empty() {
            return Self::empty();
        }

        let mut result = [0; HV_DIM_U64];
        let threshold = inputs.len() / 2;

        // Iterate over each bit position (0 to 10047)
        for bit_idx in 0..HV_DIM_BITS {
            let u64_idx = bit_idx / 64;
            let bit_offset = bit_idx % 64;
            
            let mut count = 0;
            for input in inputs {
                if (input.bits[u64_idx] >> bit_offset) & 1 == 1 {
                    count += 1;
                }
            }

            if count > threshold {
                result[u64_idx] |= 1 << bit_offset;
            }
        }

        Self { bits: result }
    }

    /// Normalized Hamming Distance Similarity (0.0 to 1.0).
    /// 1.0 means identical, 0.0 means completely opposite (all bits flipped), 0.5 means orthogonal.
    pub fn similarity(&self, other: &Hypervector) -> f32 {
        let mut total_hamming_distance = 0;
        for i in 0..HV_DIM_U64 {
            total_hamming_distance += (self.bits[i] ^ other.bits[i]).count_ones();
        }
        
        1.0 - (total_hamming_distance as f32 / HV_DIM_BITS as f32)
    }

    /// Local Sensitive Hashing (LSH) projection from dense vector.
    pub fn project(dense_vector: &[f32]) -> Hypervector {
        let mut result = [0; HV_DIM_U64];

        for bit_idx in 0..HV_DIM_BITS {
            // Seed RNG with bit index for determinism
            let mut rng = StdRng::seed_from_u64(bit_idx as u64);
            
            // Generate random vector R_i and compute dot product
            let mut dot_product = 0.0;
            for &val in dense_vector {
                // Generate random weight in [-1.0, 1.0]
                let weight: f32 = rng.random_range(-1.0..1.0);
                dot_product += val * weight;
            }

            if dot_product > 0.0 {
                let u64_idx = bit_idx / 64;
                let bit_offset = bit_idx % 64;
                result[u64_idx] |= 1 << bit_offset;
            }
        }

        Self { bits: result }
    }

    /// Faster projection using pre-computed matrix
    pub fn project_with_matrix(dense_vector: &[f32], matrix: &ProjectionMatrix) -> Hypervector {
        let mut result = [0; HV_DIM_U64];

        for bit_idx in 0..HV_DIM_BITS {
            let weights = &matrix.weights[bit_idx];
            
            // Compute dot product
            let mut dot_product = 0.0;
            for (i, &val) in dense_vector.iter().enumerate() {
                if i < weights.len() {
                    dot_product += val * weights[i];
                }
            }

            if dot_product > 0.0 {
                let u64_idx = bit_idx / 64;
                let bit_offset = bit_idx % 64;
                result[u64_idx] |= 1 << bit_offset;
            }
        }

        Self { bits: result }
    }

    /// Weighted bundle update (Hebbian Learning).
    pub fn update(&mut self, new_info: &Hypervector, weight: f32) {
        // Create a list of vectors for bundling
        // 1 copy of self
        // k copies of new_info
        
        let k = (weight * 10.0).round() as usize;
        if k == 0 {
            return; // No update if weight is too small
        }

        let mut inputs = Vec::with_capacity(1 + k);
        inputs.push(*self);
        for _ in 0..k {
            inputs.push(*new_info);
        }

        *self = Self::bundle(&inputs);
    }

    pub fn compound(op: &Operator, args: &[Hypervector]) -> Self {
        let mut inputs = Vec::new();
        
        // Operator vector
        let op_str = format!("{:?}", op);
        let id = deterministic_hash(&op_str);
        let mut rng = StdRng::seed_from_u64(id);
        let mut bits = [0; HV_DIM_U64];
        for i in 0..HV_DIM_U64 {
            bits[i] = rng.random();
        }
        inputs.push(Self { bits });

        for arg in args {
            inputs.push(*arg);
        }
        
        Self::bundle(&inputs)
    }

    pub fn from_term(term: &Term) -> Self {
        match term {
            Term::Atom(s) => {
                let id = deterministic_hash(s);
                let mut rng = StdRng::seed_from_u64(id);
                let mut bits = [0; HV_DIM_U64];
                for i in 0..HV_DIM_U64 {
                    bits[i] = rng.random();
                }
                Self { bits }
            },
            Term::Var(_, s) => {
                 let id = deterministic_hash(s);
                 let mut rng = StdRng::seed_from_u64(id);
                 let mut bits = [0; HV_DIM_U64];
                 for i in 0..HV_DIM_U64 {
                     bits[i] = rng.random();
                 }
                 Self { bits }
            },
            Term::Compound(op, args) => {
                let mut inputs = Vec::new();
                
                // Operator vector
                let mut hasher = DefaultHasher::new();
                op.hash(&mut hasher);
                let op_hash = hasher.finish();
                let mut rng = StdRng::seed_from_u64(op_hash);
                let mut op_bits = [0; HV_DIM_U64];
                for i in 0..HV_DIM_U64 {
                    op_bits[i] = rng.random();
                }
                inputs.push(Hypervector { bits: op_bits });

                for arg in args {
                    inputs.push(Self::from_term(arg));
                }
                
                // Ensure odd number of inputs for better bundling properties
                if inputs.len() % 2 == 0 {
                    let mut rng = StdRng::seed_from_u64(99999); // Constant seed
                    let mut bias_bits = [0; HV_DIM_U64];
                    for i in 0..HV_DIM_U64 {
                        bias_bits[i] = rng.random();
                    }
                    inputs.push(Hypervector { bits: bias_bits });
                }

                Self::bundle(&inputs)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub term: Term,
    pub vector: Hypervector,
    pub priority: f32,
    pub durability: f32,
    pub truth: TruthValue,
    pub stamp: Stamp,
    pub beliefs: Vec<Sentence>,
}

impl Concept {
    pub fn new(term: Term, vector: Hypervector, truth: TruthValue, stamp: Stamp) -> Self {
        Self {
            term,
            vector,
            priority: 0.5, // Default
            durability: 0.5, // Default
            truth,
            stamp,
            beliefs: Vec::new(),
        }
    }

    pub fn add_belief(&mut self, belief: Sentence) {
        // Check if belief already exists (by stamp or content) to avoid duplicates?
        // For now, just add it as requested.
        // Maybe limit the size of beliefs?
        if self.beliefs.len() > 100 {
            self.beliefs.remove(0);
        }
        self.beliefs.push(belief);
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConceptStore {
    pub map: HashMap<Term, Concept>,
    #[serde(skip)] // Bag is rebuilt on load (or transient)
    pub priority_bag: Bag<Term>, 
    pub capacity: usize,
}

impl ConceptStore {
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            priority_bag: Bag::new(capacity),
            capacity,
        }
    }

    pub fn put(&mut self, concept: Concept) {
        // 1. Evict if needed (only if adding a NEW key)
        if !self.map.contains_key(&concept.term) && self.map.len() >= self.capacity {
            self.forget_weakest();
        }

        // 2. Update Priority Bag
        // Utility = P * D (Stability)
        let utility = (concept.priority * concept.durability).clamp(0.01, 0.99);
        self.priority_bag.put(concept.term.clone(), utility);

        // 3. Update Storage
        self.map.insert(concept.term.clone(), concept);
    }

    pub fn get(&self, term: &Term) -> Option<&Concept> {
        self.map.get(term)
    }
    
    pub fn get_mut(&mut self, term: &Term) -> Option<&mut Concept> {
        self.map.get_mut(term)
    }
    
    pub fn values(&self) -> std::collections::hash_map::Values<Term, Concept> {
        self.map.values()
    }
    
    pub fn keys(&self) -> std::collections::hash_map::Keys<Term, Concept> {
        self.map.keys()
    }
    
    pub fn len(&self) -> usize {
        self.map.len()
    }

    fn forget_weakest(&mut self) {
        if let Some(weak_term) = self.priority_bag.take_weakest() {
            self.map.remove(&weak_term);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_accumulation() {
        // 1. Create two random vectors: Tiger and Feline
        let mut tiger = Hypervector::random();
        let feline = Hypervector::random();

        // 2. Assert similarity is approx 0.5 (random orthogonality)
        let initial_sim = tiger.similarity(&feline);
        println!("Initial Similarity: {}", initial_sim);
        assert!((initial_sim - 0.5).abs() < 0.1, "Random vectors should be approx orthogonal (0.5 similarity)");

        // 3. Update Tiger with Feline (simulating <Tiger --> Feline>)
        // Using a weight of 0.5 (so k=5 copies of Feline vs 1 copy of Tiger)
        tiger.update(&feline, 0.5);

        // 4. Assert similarity has increased significantly
        let new_sim = tiger.similarity(&feline);
        println!("New Similarity: {}", new_sim);
        assert!(new_sim > initial_sim + 0.1, "Similarity should increase after update");
        assert!(new_sim > 0.6, "Similarity should be significant");
    }

    #[test]
    fn test_bind_inverse() {
        let a = Hypervector::random();
        let b = Hypervector::random();
        
        let bound = a.bind(&b);
        let unbound = bound.bind(&b); // XOR is its own inverse
        
        assert_eq!(a, unbound, "XOR binding should be reversible");
    }

    #[test]
    fn test_bundle_majority() {
        let a = Hypervector::random();
        let b = Hypervector::random();
        let c = Hypervector::random();
        
        // Create a bundle where 'a' appears 3 times, 'b' 1 time, 'c' 1 time.
        // 'a' should dominate.
        let inputs = vec![a, a, a, b, c];
        let bundled = Hypervector::bundle(&inputs);
        
        let sim_a = bundled.similarity(&a);
        let sim_b = bundled.similarity(&b);
        
        assert!(sim_a > sim_b, "Majority element should be more similar to bundle");
        assert!(sim_a > 0.8, "Bundle should be very similar to dominant element");
    }
}
