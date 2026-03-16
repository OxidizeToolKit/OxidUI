# Contributing to TermOxide

Welcome! We are excited that you want to contribute to TermOxide.
This document outlines our process for development, from creating issues to submitting Pull Requests, to ensure a smooth and consistent workflow.

## Table of Contents

1. [Workflow Overview](#1-workflow-overview)
2. [Development Setup](#2-development-setup)
3. [Branching Strategy](#3-branching-strategy)
4. [Commit Rules](#4-commit-rules)
5. [Git Hooks & Formatting](#5-git-hooks--formatting)
6. [Code Style & Testing](#6-code-style--testing)
7. [Issues Guidelines](#7-issues-guidelines)
8. [Pull Request Process](#8-pull-request-process)

---

## 1. Workflow Overview

We use a GitHub Flow with protected branches, strict commit rules, and mandatory PR reviews.
External contributors should fork the repository and submit a Pull Request to our `main` branch.

All contributions must follow the rules below.

---

## 2. Development Setup

To start contributing, you need to set up the Rust toolchain:

1. **Install Rust and Cargo**:
   Follow the instructions on [rustup.rs](https://rustup.rs/) to install the latest stable version of Rust.

2. **Clone the repository**:

   ```sh
   git clone https://github.com/TermOxide/TermOxide.git
   cd TermOxide
   ```

3. **Install Git Hooks**:
   We use custom git hooks to enforce formatting and linting.
   Make sure to run the setup script:
   - Linux/macOS: [./.githooks/setup-hooks.sh](./.githooks/setup-hooks.sh)
   - Windows: [./.githooks/setup-hooks.ps1](./.githooks/setup-hooks.ps1)

4. **Verify your setup**:

   ```sh
   cargo build
   cargo test
   ```

---

## 3. Branching Strategy

### Main Branch

| Branch | Purpose |
| ------ | ------- |
| `main` | Primary branch. Protected, no direct pushes. All PRs target this branch. |

### Development Branches

Members of the project can work on branches starting with `dev/`.
*Note: Git hooks (like [pre-commit](./.githooks/pre-commit)) bypass checks on `dev/*` branches to allow fast iteration locally.*

External contributors working from a fork can use any branch name, but we recommend descriptive names.

---

## 4. Commit Rules

We strictly enforce a Direct-Scope Conventional Commit format using our custom `commit-msg` hook.

**Pattern:**

```txt
<scope>: <subject>
```

**Critical Rules:**

1. **All lowercase** for both the scope and the subject.
2. **No ending period** and NO HTML/XML tags.
3. **No generic prefixes** (e.g., do not use `feat:`, `fix:`, `chore:`). The prefix MUST be the scope itself.

**Scope Logic:**

- Dynamically infer the `<scope>` from the directory structure, the crate name, or the workspace path of the staged files. (e.g., if files are in `crates/react/*`, the scope is `react`).
- If changes relate to formatting, UI styling, or layout, strictly use `style` as the scope (NEVER `styling`).
- For CI/CD configuration files, use `ci/build`.
- If multiple scopes are touched, use the highest common denominator or the primary crate affected.

**Subject Logic:**

- Precise description of the functional change in English.
- Use the **Imperative Mood** (e.g., "add", "fix", "implement").

**Examples:**

- `bootstrap: initialize cargo project`
- `ci/build: compute all crate that need to be rebuild`
- `style: implement a recursive tree in taffy for the layout`
- `react: fix the termoxide-react crate config files`

---

## 5. Git Hooks & Formatting

We enforce quality checks locally using Git Hooks before you can commit.

The [pre-commit](./.githooks/pre-commit) hook automatically runs:

1. `cargo fmt --all -- --check`
2. `cargo check --all-targets --all-features`
3. `cargo clippy --all-targets --all-features -- -D warnings`

If any of these fail, your commit will be rejected. Fix the errors and try committing again.
*(Reminder: these are bypassed if your branch starts with `dev/`)*

---

## 6. Code Style & Testing

Our CI pipeline will verify these requirements, so ensure you meet them locally:

- **Formatting:** All code must be formatted using `cargo fmt`.
- **Linting:** Code must pass `cargo clippy` without any warnings (`-D warnings`).
- **Testing Coverage:** We aim for >70% test coverage on our core framework. Ensure you write tests for new components or reactive primitives you add.

---

## 7. Issues Guidelines

**You must always create an issue before opening a PR**, and link your commits/PR to this issue (e.g., `Closes #12`).

When creating an issue, make sure to apply the appropriate labels from our standard taxonomy:

### Label Taxonomy

**Releases & Status**

- `00.release: major`: Breaking changes requiring a major version bump (x.0.0)
- `00.release: minor`: New features requiring a minor version bump (0.x.0)
- `00.release: patch`: Bug fixes and small improvements requiring a patch version bump (0.0.x)
- `01.status: blocked`: Blocked by another issue/PR
- `01.status: changes-requested`: Reviewers have requested changes
- `01.status: merge-conflict`: PR has merge conflicts
- `01.status: ready-for-merge`: PR is approved and ready
- `01.status: waiting-for-review`: PR is awaiting reviewer feedback

**Hierarchy (Layer)**

- `02.layer: epic`: High-level theme grouping related stories and tasks
- `02.layer: story`: Roadmap requirement essential for project functionality
- `02.layer: task`: A unit of work required to complete a story

**Priority**

- `03.priority: critical`: Immediate action needed — blocks progress/major risk
- `03.priority: high`: Requires quick attention to prevent delays
- `03.priority: medium`: Important but not urgent
- `03.priority: low`: Small improvement to address later
- `03.priority: bonus`: Nice-to-have, non-mandatory addition

**Size Estimations**

- `04.size: XS`, `04.size: S`, `04.size: M`, `04.size: L`, `04.size: XL`

**Type**

- `1.type: bug`
- `1.type: build`
- `1.type: ci`
- `1.type: cleanup`
- `1.type: dependencies`
- `1.type: documentation`
- `1.type: extra-deliverable`
- `1.type: feature`
- `1.type: refactor`
- `1.type: regression`
- `1.type: test`

**Component**

- `2.component: uncategorized`: Needs categorization

---

## 8. Pull Request Process

1. **Create an issue** and apply relevant priority, type, and layer labels.
2. **Create a branch** (or fork if you are an external contributor).
3. **Commit your changes** adhering perfectly to the [Commit Rules](#4-commit-rules).
4. **Push your work** and open a Pull Request against the `main` branch.
   - Apply any existing PR template if prompted.
   - Link the PR to the issue (`Closes #XX`).
5. **Pass all CI checks** and local git hooks.
6. **Get reviews:** At least **20% of the project maintainers** must approve the PR.
7. **Merge:** Once approved and CI is green, a maintainer will merge your PR.
