# Binix Browser

Un navigateur web ultra-performant écrit en Rust.

## Objectifs
- Léger & rapide
- Sécurité mémoire
- Architecture modulaire

## Build
```bash
cargo build --release -p binix-app
```

## Structure
Workspace avec crates modulaires (core, net, dom, css, ...)

Voir `ARCHITECTURE.md` pour détails.