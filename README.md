# ternary-gate

Ternary gating for mixture-of-experts (MoE) routing. Each expert receives a ternary decision — **{−1 = block, 0 = skip, +1 = activate}** — enabling sparse activation that reduces both compute and memory by up to 16× compared to dense expert evaluation.

## Why It Matters

Mixture-of-experts models (Shazeer et al., 2017) achieve trillion-parameter scale by activating only a subset of experts per token. Standard MoE uses soft gating (continuous weights), requiring all experts to be loaded into memory even when most are inactive.

Ternary gating transforms this into a **hard decision**:

| Gate | Semantic | Action | Cost |
|------|----------|--------|------|
| +1 | Activate | Run full expert forward pass | O(d²) |
| 0 | Skip | Do nothing, contribute nothing | O(1) |
| −1 | Block | Explicitly exclude (negative signal) | O(1) |

Only the `+1` experts execute, giving a theoretical speedup of **N/k** where N = total experts and k = activated experts.

## How It Works

### Routing Algorithm

Given an input vector x and N experts with scores s₁, ..., sₙ:

```
1. Sort experts by score (descending)
2. Top-k experts with score > 0 → Gate::Activate (+1)
3. Experts with score < −0.5     → Gate::Block (−1)
4. All others                     → Gate::Skip (0)
```

The threshold −0.5 for blocking is configurable and represents a "strong negative signal" — the expert is not just irrelevant but actively harmful for this input.

### Load Balancing

Optimal MoE routing requires uniform expert utilization. The load balance metric is:

```
L = 1 − Σᵢ |fᵢ − 1/N|
```

where fᵢ = (activations of expert i) / (total activations). A perfectly balanced router has L = 1.0; a router that always picks the same expert has L approaching 0.

### Sparse Forward Pass

Only activated experts process the input:

```
y = Σᵢ∈{active}  fᵢ(x)
```

This is a **conditional computation graph** — inactive experts require zero FLOPs. With N = 64 experts and k = 2 active, the compute savings are 32×.

### Complexity

| Operation | Time | Space |
|-----------|------|-------|
| `route(scores)` | O(N log N) | O(N) |
| `sparse_forward(input, gates)` | O(k·d) | O(k·d) |
| `load_balance()` | O(N) | O(1) |
| `most_active()` | O(N) | O(1) |

Where N = number of experts, k = top-k activated, d = input dimension.

### Comparison with Dense MoE

| Approach | Memory | Compute | Latency |
|----------|--------|---------|---------|
| Dense (all experts) | O(N·d²) | O(N·d²) | High |
| Soft top-k (standard MoE) | O(N·d²) | O(k·d²) | Medium |
| **Ternary gate (this crate)** | **O(N·d²) loaded, O(k·d²) compute** | **O(k·d²)** | **Low** |

The key insight: ternary gating makes the skip explicit (Gate::Skip), enabling hardware-level predicated execution where skipped experts don't even fetch weights.

## Quick Start

```rust
use ternary_gate::{TernaryGateRouter, Gate};

let mut router = TernaryGateRouter::new(2); // top-2 activation
router.add_expert("attention");
router.add_expert("feedforward");
router.add_expert("convolution");
router.add_expert("embedding");

// Route based on input scores
let scores = vec![0.9, -0.8, 0.3, -0.1];
let result = router.route(&scores);

println!("Active: {:?}", result.active_experts);
println!("Blocked: {:?}", result.blocked_experts);
println!("Skipped: {:?}", result.skipped_experts);

// Check load balance
println!("Balance: {:.3}", router.load_balance());

// Sparse forward — only active experts compute
let input = vec![1, -1, 0, 1];
let outputs = router.sparse_forward(&input, &result);
```

## API

### `TernaryGateRouter`

| Method | Description |
|--------|-------------|
| `new(top_k)` | Create router activating top-k experts |
| `add_expert(name)` | Register a new expert |
| `route(scores) -> GateResult` | Score-based ternary routing |
| `sparse_forward(input, gates)` | Execute only active experts |
| `load_balance() -> f64` | Balance metric ∈ [0, 1] |
| `most_active() -> &Expert` | Expert with highest activation count |
| `expert_count() / routing_count()` | Statistics |

### `Gate`

```rust
pub enum Gate {
    Block = -1,    // Explicitly excluded
    Skip = 0,      // Not selected (neutral)
    Activate = 1,  // Will compute
}
```

## Architecture Notes

This crate implements the **γ (gamma) control layer** of the γ + η = C framework for mixture-of-experts:

- **γ (gamma)**: The gating/routing logic — deciding which experts to activate. This crate provides ternary γ-level routing decisions.
- **η (eta)**: The expert computation — the actual neural network forward passes performed by activated experts. Provided by ecosystem inference crates.
- **C**: The complete MoE inference system. γ decides who runs; η does the running.

The ternary gate values {−1, 0, +1} are the same domain used throughout the ecosystem for ternary weights, enabling seamless integration with ternary-quantized experts.

## References

- **Mixture of Experts**: Jacobs, R.A. et al., "Adaptive Mixtures of Local Experts," Neural Computation, 3(1), 79-87, 1991.
- **Sparsely-Gated MoE**: Shazeer, N. et al., "Outrageously Large Neural Networks: The Sparsely-Gated Mixture-of-Experts Layer," ICLR 2017.
- **GShard**: Lepikhin, D. et al., "GShard: Scaling Giant Models with Conditional Computation and Automatic Sharding," ICLR 2021.
- **Switch Transformers**: Fedus, W. et al., "Switch Transformers: Scaling to Trillion Parameter Models with Simple and Efficient Sparsity," JMLR, 2022.
- **Expert Choice Routing**: Zhou, Y. et al., "Brainformers: Trading Simplicity for Efficiency," 2022.

## License

MIT
