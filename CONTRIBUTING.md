# Contributing

Thanks for considering a contribution.

## Before opening a pull request

1. Keep numerical geometry independent of Plotters backends.
2. Preserve semantic A/B/C composition order and invisible mathematical viewport clipping.
3. Add focused tests for numerical changes and regenerate intentional reference artifacts.
4. Run:

   ```text
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo test --no-default-features
   ```

5. Update architecture notes or an ADR when public semantics change.

Do not introduce unsupported scope—filled contours, arbitrary triangulation,
Kuhn simplices, N-component grids, or bindings—without a separately agreed
design milestone.
