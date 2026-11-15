# Validação completa, na ordem do AGENTS.md ("Verificação automática").
# O cargo test vem antes do frontend porque regenera os bindings do ts-rs.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    npx eslint .
    npx tsc -b
    npx vitest run

# Formata o código Rust.
fmt:
    cargo fmt --all

# Abre o app em modo de desenvolvimento.
dev:
    npm run tauri dev
