use rand::Rng;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug)]
pub struct Bag<T: Clone + Eq + Hash> {
    pub levels: Vec<Vec<T>>, // 100 levels of priority (0.00 to 0.99)
    pub capacity: usize,
    pub count: usize,
    pub name_map: HashMap<T, f32>, // item -> priority
}

impl<T: Clone + Eq + Hash> Bag<T> {
    pub fn new(capacity: usize) -> Self {
        let mut levels = Vec::with_capacity(100);
        for _ in 0..100 { levels.push(Vec::new()); }
        Self { levels, capacity, count: 0, name_map: HashMap::new() }
    }

    pub fn put(&mut self, item: T, priority: f32) {
        // If exists, remove old version first (update)
        if self.name_map.contains_key(&item) {
            self.take_specific(&item);
        }
        
        // Evict if full
        if self.count >= self.capacity {
            self.evict_weakest();
        }

        // Insert new
        let level = (priority * 99.0).clamp(0.0, 99.0) as usize;
        self.levels[level].push(item.clone());
        self.name_map.insert(item, priority);
        self.count += 1;
    }

    pub fn take(&mut self) -> Option<T> {
        if self.count == 0 { return None; }
        
        let mut rng = rand::rng();
        // Probabilistic selection: Bias towards top levels
        // Try 3 times to pick a non-empty level biased towards 100
        for _ in 0..3 {
            let r = rng.random_range(0..100);
            let level = 99 - (r * r / 100); // Quadratic bias
            
            if !self.levels[level].is_empty() {
                let idx = rng.random_range(0..self.levels[level].len());
                let item = self.levels[level].remove(idx);
                self.name_map.remove(&item);
                self.count -= 1;
                return Some(item);
            }
        }
        
        // Fallback: strict scan from top down to find *any* item
        for level in (0..100).rev() {
            if !self.levels[level].is_empty() {
                 let item = self.levels[level].remove(0);
                 self.name_map.remove(&item);
                 self.count -= 1;
                 return Some(item);
            }
        }
        None
    }
    
    fn take_specific(&mut self, item: &T) {
        if let Some(&p) = self.name_map.get(item) {
            let level = (p * 99.0).clamp(0.0, 99.0) as usize;
            if let Some(pos) = self.levels[level].iter().position(|x| x == item) {
                self.levels[level].remove(pos);
                self.name_map.remove(item);
                self.count -= 1;
            }
        }
    }

    // Remove weakest item (for eviction)
    fn evict_weakest(&mut self) {
        for level in 0..100 {
            if !self.levels[level].is_empty() {
                let item = self.levels[level].remove(0); // FIFO in lowest bucket
                self.name_map.remove(&item);
                self.count -= 1;
                return;
            }
        }
    }
    
    // For ConceptStore eviction (public helper)
    pub fn take_weakest(&mut self) -> Option<T> {
        for level in 0..100 {
            if !self.levels[level].is_empty() {
                let item = self.levels[level].remove(0);
                self.name_map.remove(&item);
                self.count -= 1;
                return Some(item);
            }
        }
        None
    }
}

impl<T: Clone + Eq + Hash> Default for Bag<T> {
    fn default() -> Self {
        Self::new(100)
    }
}
