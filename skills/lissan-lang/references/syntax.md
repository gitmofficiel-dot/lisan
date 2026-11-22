# Lissan reference pointers

Full language rules live in the repository docs. Read these when the cheat sheet in SKILL.md is not enough:

- `docs/SYNTAX.md`: values, statements, standard library, loops, exceptions, match.
- `docs/NUMBERS-AND-STYLE.md`: numeral systems and code style.
- `docs/EXECUTION-PATHS.md`: interpreter, VM and native paths.
- `README.md`: install, hardware modules, tooling.
- `examples/`: runnable programs for every feature.

Source of truth for keywords: `src/lexer.rs` (the keyword match table). When this skill and the lexer disagree, the lexer wins.
