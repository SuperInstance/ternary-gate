//! # ternary-gate
//!
//! Ternary gating for mixture-of-experts on GPU.
//! Each expert gets {-1=block, 0=skip, +1=activate}.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate { Block = -1, Skip = 0, Activate = 1 }
impl Gate { pub fn val(&self) -> i8 { *self as i8 } }

#[derive(Debug, Clone)]
pub struct Expert {
    pub id: u32,
    pub name: String,
    pub gate: Gate,
    pub activation_count: u64,
}

impl Expert {
    pub fn new(id: u32, name: &str) -> Self {
        Self { id, name: name.into(), gate: Gate::Skip, activation_count: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct GateResult {
    pub active_experts: Vec<u32>,
    pub blocked_experts: Vec<u32>,
    pub skipped_experts: Vec<u32>,
}

pub struct TernaryGateRouter {
    experts: Vec<Expert>,
    top_k: usize,
    routing_history: Vec<GateResult>,
}

impl TernaryGateRouter {
    pub fn new(top_k: usize) -> Self {
        Self { experts: Vec::new(), top_k, routing_history: Vec::new() }
    }

    pub fn add_expert(&mut self, name: &str) {
        let id = self.experts.len() as u32;
        self.experts.push(Expert::new(id, name));
    }

    /// Route input by scoring experts and gating top-k.
    pub fn route(&mut self, input_scores: &[f64]) -> GateResult {
        let mut scored: Vec<(usize, f64)> = input_scores.iter().enumerate()
            .map(|(i, &s)| (i, s)).collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let top_k = self.top_k.min(self.experts.len());
        let mut active = vec![];
        let mut blocked = vec![];
        let mut skipped = vec![];

        for (rank, &(idx, score)) in scored.iter().enumerate() {
            if idx >= self.experts.len() { continue; }
            let expert = &mut self.experts[idx];
            if rank < top_k && score > 0.0 {
                expert.gate = Gate::Activate;
                expert.activation_count += 1;
                active.push(expert.id);
            } else if score < -0.5 {
                expert.gate = Gate::Block;
                blocked.push(expert.id);
            } else {
                expert.gate = Gate::Skip;
                skipped.push(expert.id);
            }
        }

        let result = GateResult { active_experts: active, blocked_experts: blocked, skipped_experts: skipped };
        self.routing_history.push(result.clone());
        result
    }

    /// Sparse activation: only active experts process.
    pub fn sparse_forward(&self, input: &[i8], gates: &GateResult) -> Vec<Vec<i8>> {
        gates.active_experts.iter().map(|&id| {
            let expert = &self.experts[id as usize];
            // Simplified: each expert transforms input differently
            input.iter().map(|&v| {
                match expert.gate {
                    Gate::Activate => v,
                    Gate::Block => 0,
                    Gate::Skip => 0,
                }
            }).collect()
        }).collect()
    }

    pub fn expert_count(&self) -> usize { self.experts.len() }
    pub fn routing_count(&self) -> usize { self.routing_history.len() }
    pub fn most_active(&self) -> Option<&Expert> {
        self.experts.iter().max_by_key(|e| e.activation_count)
    }

    /// Load balance: check if activations are evenly distributed.
    pub fn load_balance(&self) -> f64 {
        if self.experts.is_empty() { return 1.0; }
        let total: u64 = self.experts.iter().map(|e| e.activation_count).sum();
        if total == 0 { return 1.0; }
        let ideal = 1.0 / self.experts.len() as f64;
        let imbalance: f64 = self.experts.iter()
            .map(|e| ((e.activation_count as f64 / total as f64) - ideal).abs())
            .sum();
        1.0 - imbalance // 1.0 = perfectly balanced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_routing() {
        let mut router = TernaryGateRouter::new(2);
        router.add_expert("expert_a");
        router.add_expert("expert_b");
        router.add_expert("expert_c");
        let result = router.route(&[0.9, 0.5, -0.8]);
        assert_eq!(result.active_experts.len(), 2);
        assert!(result.blocked_experts.contains(&2)); // expert_c blocked
    }

    #[test]
    fn test_top_k_routing() {
        let mut router = TernaryGateRouter::new(1);
        for i in 0..5 { router.add_expert(&format!("e{}", i)); }
        let result = router.route(&[0.1, 0.9, 0.3, 0.2, 0.1]);
        assert_eq!(result.active_experts.len(), 1);
        assert_eq!(result.active_experts[0], 1); // highest score
    }

    #[test]
    fn test_sparse_forward() {
        let mut router = TernaryGateRouter::new(2);
        router.add_expert("a"); router.add_expert("b");
        let result = router.route(&[0.8, 0.6]);
        let input = vec![1, -1, 0, 1];
        let outputs = router.sparse_forward(&input, &result);
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_most_active() {
        let mut router = TernaryGateRouter::new(1);
        router.add_expert("a"); router.add_expert("b");
        router.route(&[0.9, 0.1]); // a wins
        router.route(&[0.9, 0.1]); // a wins again
        assert_eq!(router.most_active().unwrap().name, "a");
    }

    #[test]
    fn test_load_balance() {
        let mut router = TernaryGateRouter::new(1);
        router.add_expert("a"); router.add_expert("b");
        router.route(&[0.9, 0.1]);
        assert!(router.load_balance() < 1.0); // imbalanced
    }

    #[test]
    fn test_routing_history() {
        let mut router = TernaryGateRouter::new(2);
        router.add_expert("a"); router.add_expert("b");
        router.route(&[0.5, 0.3]);
        router.route(&[0.1, 0.9]);
        assert_eq!(router.routing_count(), 2);
    }

    #[test]
    fn test_block_negative() {
        let mut router = TernaryGateRouter::new(1);
        router.add_expert("a"); router.add_expert("b");
        let result = router.route(&[0.9, -0.8]);
        assert!(result.blocked_experts.contains(&1));
    }
}
