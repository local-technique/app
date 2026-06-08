# Agent Rules

Never commit, amend, or push changes unless the user explicitly asks for that exact git action.

After any code change, run CI-equivalent checks:

- Frontend (`front/`): `npm run lint`, `npm run test`, `npm run build`
- Backend (`back/`): `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, `cargo build --all-features`
- For dependency changes in any ecosystem, verify the latest documented version and supported config/inputs before editing; do not rely on memory.
- Do not use older dependency versions unless user explicitly requests it or a documented incompatibility is cited.
- No guessing protocol: if documentation is unclear after checking, stop and ask one targeted question with links.
