# Contributing to lambda-appsync

🎉 First off, thanks for taking the time to contribute! We truly appreciate your support and interest in improving `lambda-appsync`.

Whether you're here to report bugs, suggest features, improve documentation, or submit code – you're in the right place.

---

## 🧰 Project Structure

This repository is a Rust workspace with two crates:

- **`lambda-appsync`** – The main crate: exposes types and runtime integration for AWS AppSync Direct Lambda resolvers.
- **`lambda-appsync-proc`** – Procedural macro crate: generates types and boilerplate based on GraphQL schemas.

---

## 🗂️ How to Contribute

### 🐛 Bug Reports

If you’ve found something that doesn’t look right:

1. **Search existing issues** to see if it's already been reported.
2. If not, [open a new issue](https://github.com/JeremieRodon/lambda-appsync/issues/new) and provide:
   - A clear title and description.
   - Steps to reproduce the issue.
   - Expected vs actual behavior.
   - Minimal code example or schema.

### 💡 Feature Requests

We welcome ideas that can improve the developer experience or extend functionality. When opening a feature request:

- Describe the problem you're trying to solve.
- Explain how your suggestion addresses it.
- Optionally, suggest an implementation path.

### 🛠️ Code Contributions

We welcome PRs for bug fixes, features, performance improvements, or refactors.

#### 1. Fork the Repo

```sh
git clone https://github.com/YOUR_USERNAME/lambda-appsync.git
cd lambda-appsync
```

#### 2. Set Up the Project

```sh
cargo check
cargo test
```

Make sure `graphql-parser` and other dependencies are working correctly. You’ll need Rust 1.81+.

#### 3. Make Your Changes

- Write clean, idiomatic Rust.
- Follow the existing code style (run `cargo fmt`).
- Add/update tests if relevant (`cargo test`).
- Document new public APIs using `///` comments.
- Validate changes using an example schema in `examples/` or your own test crate.

#### 4. Run the Full Test Suite

```sh
cargo test --workspace
```

#### 5. Submit the Pull Request

- Describe what you’ve changed and why.
- Link the issue you’re fixing if applicable.
- Be open to feedback and iteration.

---

## ✅ Code Quality Checklist

Before submitting a PR:

- [ ] Tests pass
- [ ] `cargo fmt` ran successfully
- [ ] `cargo clippy` has no critical warnings
- [ ] Added/updated documentation where necessary
- [ ] Feature/bugfix is covered by tests (if applicable)

---

## 📦 Releasing

Only maintainers can publish new versions. When a release is due:

- Bump the version in `workspace.package.version` in `Cargo.toml`.
- Tag the release commit.
- Update the changelog (if maintained separately).

---

## 💬 Community & Support

Have questions? Want to discuss implementation strategies or architecture? You can:

- Open a [GitHub Discussion](https://github.com/JeremieRodon/lambda-appsync/discussions)
- Comment on open issues/PRs

---

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thanks again for being part of this project ❤️
