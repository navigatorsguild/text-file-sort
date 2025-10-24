# Maintenance Workflow for Rust Crate Repositories

This document provides step-by-step instructions for performing comprehensive maintenance on a Rust crate repository, including standardizing project files, updating dependencies, fixing linter warnings, and publishing a new version.

## Prerequisites

- Repository must be a Rust crate (have `Cargo.toml`)
- You must have write access to the repository
- Repository must be on GitHub
- For publishing: `CARGO_REGISTRY_TOKEN` must be configured in GitHub repository or organization secrets

## Reference Repository

This workflow was successfully completed on `command-executor` repository. Use it as a reference:
- Repository: https://github.com/navigatorsguild/command-executor
- Reference templates from: `/Users/giora/src/benchmark-rs/`

## Overall Task List

1. Create MAINTENANCE issue template
2. Create GitHub Action workflow to publish crates
3. Update all dependencies
4. Run clippy and fix all errors and warnings (zero warnings policy)
5. Publish the latest version of the crate

---

## Task 1: Create MAINTENANCE Issue Template and Standardize Files

### Goal
Add a MAINTENANCE issue template matching benchmark-rs format and standardize project files.

### Steps

1. **Copy MAINTENANCE template from benchmark-rs**
   - Source: `/Users/giora/src/benchmark-rs/.github/ISSUE_TEMPLATE/maintenance.md`
   - Destination: `.github/ISSUE_TEMPLATE/maintenance.md`
   - Exact content required (including assignees)

2. **Update all existing issue templates**
   - Update `assignees:` in `bug_report.md` and `feature_request.md` to `giora-kosoi-ng`

3. **Standardize .gitignore**
   - Copy format from `/Users/giora/src/benchmark-rs/.gitignore`
   - Must include:
     - Rust standard patterns with comments
     - `**/*.rs.bk` for rustfmt backups
     - `.idea` for JetBrains IDEs
     - `.claude/` for Claude Code settings

4. **Create branch, commit, push, and PR**
   - Branch naming: `maintenance/update-issue-templates-and-gitignore`
   - Commit message format: Short one-line description (e.g., "Add maintenance issue template, update assignees, standardize .gitignore")
   - No attribution footer needed in commit messages

### Verification
- All issue templates present in `.github/ISSUE_TEMPLATE/`
- All templates have `assignees: giora-kosoi-ng`
- `.gitignore` follows benchmark-rs format with comments

---

## Task 2: Create GitHub Action Workflow to Publish Crates

### Goal
Add automated workflow to publish to crates.io when version tags are pushed.

### Steps

1. **Create maintenance issue first**
   - Title: `[MAINTENANCE] Add GitHub Action workflow to publish to crates.io`
   - Body: Brief description
   - Note the issue number for commit messages

2. **Create branch BEFORE making changes**
   - Branch naming: `maintenance/add-publish-workflow`

3. **Copy publish workflow from benchmark-rs**
   - Source: `/Users/giora/src/benchmark-rs/.github/workflows/publish.yml`
   - Destination: `.github/workflows/publish.yml`
   - Exact content required

4. **Support pre-release versions**
   - The workflow should trigger on both:
     - Stable tags: `v*.*.*` (e.g., v0.1.1, v1.2.3)
     - Pre-release tags: `v*.*.*-*` (e.g., v0.0.1-alpha, v1.0.0-beta.2)
   - Update the `on.push.tags` section to include both patterns:
     ```yaml
     on:
       push:
         tags:
           - 'v*.*.*'
           - 'v*.*.*-*'
     ```

5. **Commit and push**
   - Commit message format: `[MAINTENANCE] #<issue-number> - <description>`
   - Example: `[MAINTENANCE] #3 - Add GitHub Action workflow to publish to crates.io`
   - Create PR with reference to issue: `Closes #<issue-number>`

### Verification
- Workflow file exists at `.github/workflows/publish.yml`
- Workflow triggers on both stable and pre-release version tags
- Workflow includes:
  - Version verification (tag matches Cargo.toml)
  - Test execution
  - Publish to crates.io using `CARGO_REGISTRY_TOKEN`

---

## Task 3 & 4: Update Dependencies and Fix Clippy Warnings

### Goal
Update all dependencies to latest versions and achieve zero clippy warnings.

### Steps

1. **Create maintenance issue**
   - Title: `[MAINTENANCE] Update all dependencies to latest versions`
   - Note the issue number

2. **Create branch FIRST**
   - Branch naming: `maintenance/update-dependencies`
   - **Important:** Always create branch before making changes to avoid accidental commits to main

3. **Review current dependencies**
   - Read `Cargo.toml`
   - Note current versions

4. **Update dependencies**
   - Update `[dependencies]` and `[dev-dependencies]` to latest compatible versions
   - Use flexible version specifiers (e.g., `"1.0"` instead of `"1.0.70"`)
   - Common updates:
     - `anyhow = "1.0"`
     - `crossbeam = "0.8"`
     - Check for major version updates (e.g., `reqwest 0.11` → `0.12`)

5. **Verify updates**
   - Run: `cargo build`
   - Run: `cargo test`

6. **Run clippy and fix ALL warnings**
   - Run: `cargo clippy --all-targets --all-features -- -D warnings`
   - Fix each category of warning (see Common Clippy Fixes section below)
   - Re-run clippy until zero warnings achieved

7. **Final verification**
   - Run: `cargo test` (ensure all tests still pass)
   - Run: `cargo clippy --all-targets --all-features -- -D warnings` (verify zero warnings)

8. **Commit and push**
   - Single commit with: `[MAINTENANCE] #<issue-number> - Update dependencies and fix all clippy warnings`
   - Create PR

### Common Clippy Fixes

#### 1. `doc_lazy_continuation`
**Issue:** Bullet list continuation lines need indentation
```rust
// Before
//! * item text
//! continuation line

// After
//! * item text
//!   continuation line  (add 2 spaces after //!)
```

#### 2. `len_zero`
**Issue:** Use `is_empty()` instead of `len() == 0`
```rust
// Before
if elements.len() == 0 {

// After
if elements.is_empty() {
```

#### 3. `missing_const_for_thread_local`
**Issue:** Thread local initializers should be const
```rust
// Before
thread_local! {
    static VAR: RefCell<Option<T>> = RefCell::new(None);
}

// After
thread_local! {
    static VAR: RefCell<Option<T>> = const { RefCell::new(None) };
}
```

#### 4. `unit_cmp`
**Issue:** Asserting on unit values is redundant
```rust
// Before
assert_eq!((), tp.join().unwrap());

// After
tp.join().unwrap();
```

#### 5. `assertions_on_constants`
**Issue:** Don't assert on constant booleans
```rust
// Before
assert!(true);   // Always passes
assert!(false);  // Always fails

// After
// Remove assert!(true) entirely
panic!("Explicit error message");  // Replace assert!(false)
```

#### 6. `unused_io_amount` (CRITICAL - Correctness Issue!)
**Issue:** `write()` might not write all bytes
```rust
// Before
f.write(data).expect("Failed");  // Might lose data!

// After
f.write_all(data).expect("Failed");  // Guarantees all data written
```

#### 7. `needless_question_mark`
**Issue:** Unnecessary Ok() wrapping
```rust
// Before
fn foo() -> Result<T, E> {
    Ok(bar()?)  // bar() already returns Result<T, E>
}

// After
fn foo() -> Result<T, E> {
    bar()  // Return Result directly
}
```

#### 8. `useless_conversion`
**Issue:** Unnecessary `.into_iter()` on iterators
```rust
// Before
(1..=20).into_iter().map(...)

// After
(1..=20).map(...)  // Range is already an iterator
```

#### 9. `noop_method_call`
**Issue:** Redundant `.clone()` on references
```rust
// Before
let x: &str = "hello";
foo(x.clone())  // Just copies the reference, not the string

// After
foo(x)  // Pass the reference directly
```

#### 10. `single_component_path_imports`
**Issue:** Unused imports
```rust
// Before
use hex;  // Never used in code

// After
// Remove the line entirely, or use specific items: use hex::encode;
```

### Verification
- `cargo build` succeeds
- `cargo test` passes (all tests)
- `cargo clippy --all-targets --all-features -- -D warnings` passes with zero warnings

---

## Task 5: Publish New Version

### Goal
Publish the updated crate to crates.io with a new version number.

### Version Number Decision

Follow semantic versioning (MAJOR.MINOR.PATCH):
- **Patch (0.x.Y)**: Bug fixes and internal improvements only
- **Minor (0.X.0)**: New features, dependency updates that might affect users
- **Major (X.0.0)**: Breaking changes

**Recommended approach:**
- If dependency updates include major version bumps (e.g., reqwest 0.11→0.12): Use MINOR bump
- If clippy fixes include behavior changes (e.g., `write()` → `write_all()`): Use MINOR bump
- If only internal improvements: Use PATCH bump

### Steps

1. **Determine new version number**
   - Review changes made (dependencies, clippy fixes)
   - Decide on version (e.g., 0.1.1 → 0.2.0)

2. **Create maintenance issue**
   - Title: `[MAINTENANCE] Bump version to X.Y.Z for release`
   - Note the issue number

3. **Create branch FIRST**
   - Branch naming: `maintenance/bump-version-X.Y.Z`

4. **Update Cargo.toml**
   - Change `version = "old"` to `version = "new"`

5. **Commit and push**
   - Commit: `[MAINTENANCE] #<issue-number> - Bump version to X.Y.Z`
   - Create PR
   - **Wait for PR to be merged**

6. **After PR merged: Pull main locally**
   - `git checkout main`
   - `git pull origin main`
   - Verify `Cargo.toml` shows new version

7. **Create and push version tag**
   - Create annotated tag: `git tag -a vX.Y.Z -m "Release version X.Y.Z"`
   - Push tag: `git push origin vX.Y.Z`
   - This triggers the GitHub Actions publish workflow

8. **Monitor workflow**
   - Check workflow run: `gh run list --limit 5`
   - Watch execution: `gh run watch <run-id>`
   - Verify workflow completes successfully:
     - ✓ Version verification passes
     - ✓ Tests pass
     - ✓ Publish to crates.io succeeds

### Verification
- PR merged to main with version update
- Git tag created and pushed
- GitHub Actions workflow runs successfully
- Crate appears on crates.io at new version: `https://crates.io/crates/<crate-name>`

---

## Final Test: Verify Release Works

### Goal
Test that the published crate can be downloaded and used from crates.io.

### Steps

1. **Create test directory**
   - Use: `/Users/giora/src/<crate-name>-release`
   - Initialize: `cd /Users/giora/src/<crate-name>-release && cargo init`

2. **Add dependency**
   - Edit `Cargo.toml` to add: `<crate-name> = "X.Y.Z"`
   - Add `anyhow = "1.0"` if needed

3. **Write test program**
   - In `src/main.rs`, write a simple program that:
     - Imports main types from the crate
     - Creates instances
     - Exercises key functionality
     - Prints success message

4. **Build and run**
   - Build: `cargo build --release`
   - Run: `cargo run --release`
   - Verify:
     - Crate downloads from crates.io (not local)
     - Compiles without warnings
     - Runs successfully
     - All functionality works as expected

### Verification
- Test program builds successfully
- Test program runs without errors
- Crate was downloaded from crates.io (check build output: "Downloaded <crate-name> vX.Y.Z")

---

## Important Notes

### Git Workflow
- **Always create branch BEFORE making changes** - Avoid accidental commits to main
- Use descriptive branch names: `maintenance/<description>`
- Create maintenance issues BEFORE starting work
- Use issue numbers in commit messages: `[MAINTENANCE] #<number> - <description>`

### Commit Message Format
- **Keep it simple:** One-line description of changes
- **No attribution:** Focus on the contribution, not the author
- **Format:** `[MAINTENANCE] #<issue> - <description>`
- **Example:** `[MAINTENANCE] #5 - Update dependencies and fix all clippy warnings`

### Pull Request Format
- Title matches commit message
- Body includes: `Closes #<issue-number>`
- Keep PR focused on single task

### Testing Strategy
- Run `cargo build` after each significant change
- Run `cargo test` before committing
- Run `cargo clippy --all-targets --all-features -- -D warnings` until zero warnings
- Test the published crate in a separate directory

### Common Pitfalls
1. **Forgetting to create branch first** - Always branch before making changes
2. **Incomplete clippy fixes** - Must achieve ZERO warnings, check all targets
3. **Skipping version tag** - Tag must be pushed to trigger publish workflow
4. **Wrong version format** - Tag must match pattern `v*.*.*` or `v*.*.*-*`
5. **Not waiting for crates.io** - It may take a few moments for the crate to appear after publish

---

## Interaction Pattern

When working with Claude Code:
1. Review the plan for each task before execution
2. Approve execution explicitly
3. Request detailed explanations when needed (e.g., "explain each clippy fix")
4. Verify each step completes successfully before moving to next step
5. Create branches FIRST, before any code changes

---

## Session Initialization

When starting a new session, provide this document and say:

```
I need to perform the maintenance workflow documented in MAINTENANCE_WORKFLOW.md
on this repository. Let's start with /init, then proceed through each task step
by step. I want to review and approve each major step before execution, similar
to an iterative planning approach.

Current repository: [repository path]
Reference repository: /Users/giora/src/benchmark-rs/
```

---

## Success Criteria

- [ ] MAINTENANCE issue template added
- [ ] All issue templates updated with correct assignee
- [ ] .gitignore standardized
- [ ] Publish workflow added and supports pre-release tags
- [ ] All dependencies updated
- [ ] Zero clippy warnings achieved
- [ ] New version published to crates.io
- [ ] Release verified in test project

---

## Estimated Time

- Task 1: 15-30 minutes
- Task 2: 15-30 minutes
- Task 3 & 4: 30-60 minutes (depending on number of clippy warnings)
- Task 5: 15-30 minutes
- Final test: 10-15 minutes

**Total: 1.5-3 hours per repository**

---

## Reference Links

- command-executor repository: https://github.com/navigatorsguild/command-executor
- Rust edition guide: https://doc.rust-lang.org/edition-guide/
- Semantic versioning: https://semver.org/
- Clippy lints: https://rust-lang.github.io/rust-clippy/master/
