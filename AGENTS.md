# Agent Rules

- Never commit, amend, or push changes unless the user explicitly asks for that exact git action.
- Never install new tools without the user's consent
- Use available docker engine to run tools without installing them

After any code change, run CI-equivalent checks:

- Frontend (`front/`): `npm run lint`, `npm run test`, `npm run build`
- Backend (`back/`): `cargo clippy --all-features -- -D warnings`, `cargo test --all-features`, `cargo build --all-features`
- For dependency changes in any ecosystem, verify the latest documented version and supported config/inputs before editing; do not rely on memory.
- Do not use older dependency versions unless user explicitly requests it or a documented incompatibility is cited.
- No guessing protocol: if documentation is unclear after checking, stop and ask one targeted question with links.

## Glossary

- item, most probably referring to the 3 types of items handled by the site (maintenances, incidents and projects)
  - when asked to do something for items, make sure to do this for the three types
