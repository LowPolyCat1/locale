# Contributing Guidelines

Welcome! We are thrilled that you want to contribute. Everyone is welcome here, regardless of experience level, including beginners!

---

## 1. Issue First Policy

To keep our project organized and avoid duplicate work, **every Pull Request (PR) must address an open issue**.

- **Search first:** Check the [Issue Tracker](../../issues) to see if your feature or bug is already discussed.
- **No open issue?** Please [open a new issue](../../issues/new) to describe the change or fix you plan to submit before starting your work.
- **PR requirement:** Link the corresponding issue in your PR description (e.g., `Fixes #123` or `Closes #123`).

> [!WARNING]
> Any PR opened without a corresponding issue will be **automatically closed** (unless submitted by an authorized bot). Repeatedly opening unlinked PRs may result in a permanent block from contributing to this repository.

---

## 2. Code Quality & AI Policy

We welcome the use of AI tools (such as GitHub Copilot, ChatGPT, Claude, etc.) to aid your workflow. However:

- **Quality responsibility:** You are fully responsible for the code you submit, regardless of how it was generated.
- **CI/CD checks:** Your PR must pass all CI/CD pipelines (formatting, linter, unit tests, build checks). Code that fails checks will not be merged.
- **Code review:** Ensure the code is readable, maintainable, and aligned with the rest of the codebase.

---

## 3. Getting Started

1. **Fork the repository:** Create your own copy of the repo.
2. **Clone locally:**

   ```bash
   git clone https://github.com/your-username/locale-rs.git
   cd locale-rs
   ```

3. **Create a branch:** Create a feature or bugfix branch off of `master`:

   ```bash
   git checkout -b feature/issue-number-description
   ```

4. **Make your changes:** Implement your changes and write tests if applicable.
5. **Run local checks:** Ensure all tests and linters pass locally before pushing:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test
   ```

---

## 4. Submitting a Pull Request

1. Push your branch to your fork.
2. Open a PR against our `master` branch.
3. Fill out the PR description completely:
   - State which issue this PR resolves.
   - Describe the changes made.
   - Verify that all CI/CD checks pass.
4. Respond to feedback during code review. Once approved and checks are green, your code will be merged!

---

Thank you for helping make this project better!
