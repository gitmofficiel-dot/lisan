# Contributing to Lisan | المساهمة في لسان

Thanks for helping. Small, focused changes are easiest to review.

## Quick start

```sh
git clone https://github.com/gitmofficiel-dot/lisan.git
cd lisan
cargo test --all-targets
```

## Before you open a pull request

- Run `cargo test --all-targets` and make sure it passes.
- Add or update a test for any behavior you change (see `tests/`).
- Keep docs in sync (`docs/`, `CHANGELOG.md`) when you change syntax or behavior.
- One topic per pull request. Describe what changed and why.

## Reporting bugs and ideas

Use the issue templates. For a bug, include the Lisan version, your OS, a minimal `.lsn` program and the output you expected.

Security problems: do not open a public issue. See [SECURITY.md](SECURITY.md).

## Style

- Rust code: `cargo fmt` before committing.
- Arabic and English are both welcome in issues and discussions.

## License

By contributing you agree your work is released under the [MIT License](LICENSE).

---

## بالعربية

نرحب بالمساهمات الصغيرة والواضحة. شغّل `cargo test --all-targets` قبل فتح طلب الدمج، وأضف اختبارًا لأي تغيير في السلوك، وحدّث الوثائق عند تغيير الصياغة. المشاكل الأمنية لا تُفتح علنًا، راجع [SECURITY.md](SECURITY.md).
