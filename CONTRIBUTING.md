# Contributing

Thanks for considering a contribution.

## Repository boundary

`plotters-ternary` is the rendering crate. Numerical grids, meshes, interpolation, scalar evaluation, contour topology, and band geometry belong in `ternary-contours`.

Do not create a matching long-lived Plotters branch for an in-progress `ternary-contours` milestone. Complete and merge the numerical milestone in the core repository first. Then test this crate against the merged core API and open a Plotters pull request only when rendering integration, dependency metadata, re-exports, examples, or documentation genuinely need to change.

This sequencing prevents cross-repository branch drift and avoids repeated merge conflicts.

## Before opening a pull request

1. Start from the latest `plotters-ternary/master` after the corresponding core work is merged.
2. Keep numerical geometry independent of Plotters backends.
3. Preserve semantic A/B/C composition order and invisible mathematical viewport clipping.
4. Do not merge `master` into the feature branch; rebase the branch before updating the pull request.
5. Add focused tests and regenerate intentional reference artifacts.
6. Run:

   ```text
   cargo fmt --check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --all-features
   cargo test --no-default-features
   ```

7. Update architecture notes or an ADR when public rendering semantics change.

Prefer squash merge when a branch contains corrective commits or accidental merge commits. Use rebase merge only for an already linear branch.
