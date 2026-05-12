# Agent Rules

After any code change, run CI-equivalent checks:

- Frontend (`front/`): `npm run lint`, `npm run test`, `npm run build`
- Backend (`back/`): `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, `cargo build --all-features`
