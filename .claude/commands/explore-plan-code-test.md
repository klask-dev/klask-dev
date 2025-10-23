---
description: Complete workflow for implementing new features with parallel agents execution
---

# =€ Explore-Plan-Code-Test Workflow

This command orchestrates the complete development workflow for implementing new features in Klask, using specialized AI agents in parallel for maximum efficiency.

## =Ë Workflow Steps

### 1. **Explore** =
Analyze the codebase to understand existing patterns and architecture.

**Tasks:**
- Identify relevant files in backend (`klask-rs/`) and frontend (`klask-react/`)
- Understand existing patterns and conventions
- Review similar existing features
- Identify dependencies and affected components

### 2. **Plan** =Ý
Create a detailed implementation plan.

**Tasks:**
- Break down feature into backend and frontend tasks
- Define data models and API contracts
- Plan test coverage (unit + integration)
- Identify potential challenges

### 3. **Code** =» **(PARALLEL EXECUTION)**
Implement the feature using specialized agents running in parallel.

**Backend (rust-backend-expert):**
- Implement API endpoints in `klask-rs/src/api/`
- Add business logic in `klask-rs/src/services/`
- Update data models in `klask-rs/src/models/`
- Add database migrations if needed

**Frontend (react-frontend-expert):**
- Create/modify React components in `klask-react/src/features/`
- Add API hooks in `klask-react/src/api/`
- Style with TailwindCSS
- Update TypeScript types

**Execution:**
```
Launch rust-backend-expert and react-frontend-expert agents IN PARALLEL using a single message with multiple Task tool calls.
```

### 4. **Test** >ê **(PARALLEL EXECUTION)**
Write comprehensive tests using test-specialist.

**Backend Tests (test-specialist):**
- Unit tests for business logic
- Integration tests for API endpoints
- Test error cases and edge cases

**Frontend Tests (test-specialist):**
- Component tests with React Testing Library
- Hook tests for custom hooks
- Integration tests for user flows

**Execution:**
```
Launch two test-specialist agents IN PARALLEL - one for backend, one for frontend.
```

### 5. **Debug** =
Fix any failing tests or build errors.

**Tasks:**
- Use debug-specialist if tests fail
- Analyze error messages and stack traces
- Fix issues systematically
- Re-run tests until all pass

### 6. **Review** =A
Comprehensive code review for quality, security, and performance.

**Tasks:**
- Launch code-reviewer agent
- Review all changes (backend + frontend)
- Check security vulnerabilities
- Verify best practices
- Ensure code quality standards

### 7. **Verify** 
Final verification before committing.

**Tasks:**
- Run all backend tests: `cd klask-rs && cargo test`
- Run all frontend tests: `cd klask-react && npm test`
- Check linting: `cargo clippy` and `npm run lint`
- Verify formatting: `cargo fmt --check` and formatting passes
- Ensure build succeeds: `cargo build` and `npm run build`

### 8. **Commit** =æ
Create commit with conventional commit message.

**Tasks:**
- Stage all changes
- Create commit with format: `feat: description` or `fix: description`
- **DO NOT** add "Generated with Claude Code"
- **DO NOT** add co-author

### 9. **Push & CI Check** =€
Push to GitHub and verify CI passes.

**Tasks:**
- Push to current branch or create new feature branch
- Use `gh run watch` to monitor CI execution
- Verify all GitHub Actions checks pass
- Check build, tests, and linting in CI

---

## =Ö Usage Example

```
/explore-plan-code-test Add language filter to search with dropdown in UI
```

## <¯ Implementation Instructions

When this command is invoked, execute the following:

### Step 1: Explore
Use the Glob and Grep tools to identify relevant files:
- Backend: `klask-rs/src/api/search.rs`, `klask-rs/src/services/search.rs`
- Frontend: `klask-react/src/features/search/`, `klask-react/src/api/search.ts`

### Step 2: Plan
Create a markdown plan with:
```markdown
## Backend Tasks
- [ ] Add language field to search query struct
- [ ] Update SearchService to filter by language
- [ ] Add language detection logic
- [ ] Update API endpoint response

## Frontend Tasks
- [ ] Add language dropdown to SearchFilters component
- [ ] Update search API hook to include language
- [ ] Add language to facets display
- [ ] Style dropdown with TailwindCSS

## Testing
- [ ] Backend: Test language filtering logic
- [ ] Frontend: Test language selection updates results
```

### Step 3: Code (PARALLEL)
**IMPORTANT**: Launch both agents in a SINGLE message with multiple Task tool calls:

```typescript
// Example tool usage (conceptual)
[
  Task(subagent_type="rust-backend-expert", prompt="Implement language filter in backend..."),
  Task(subagent_type="react-frontend-expert", prompt="Add language dropdown to UI...")
]
```

### Step 4: Test (PARALLEL)
Launch two test-specialist agents in parallel:

```typescript
[
  Task(subagent_type="test-specialist", prompt="Write backend tests for language filter..."),
  Task(subagent_type="test-specialist", prompt="Write frontend tests for language dropdown...")
]
```

### Step 5: Debug
If tests fail:
```typescript
Task(subagent_type="debug-specialist", prompt="Fix failing tests: [error details]")
```

### Step 6: Review
```typescript
Task(subagent_type="code-reviewer", prompt="Review all changes for language filter feature")
```

### Step 7: Verify
Run verification commands:
```bash
cd klask-rs && cargo test
cd klask-react && npm test
cargo clippy -- -D warnings
npm run lint
```

### Step 8: Commit
```bash
git add .
git commit -m "feat: add language filter to search with dropdown UI"
```

### Step 9: Push & CI Check
```bash
git push origin <current-branch>
gh run watch
```

---

## ¡ Key Requirements

1. **Parallel Execution**: ALWAYS run multiple agents in parallel when possible
   - Step 3 (Code): Backend + Frontend agents simultaneously
   - Step 4 (Test): Backend + Frontend test specialists simultaneously

2. **No Placeholders**: Never use placeholder values in tool calls

3. **Complete Implementation**: Each agent must:
   - Read existing code first
   - Follow project patterns
   - Write complete, working code
   - Include error handling

4. **Quality Gates**: Don't proceed to next step if:
   - Tests are failing
   - Build errors occur
   - Code review identifies critical issues

5. **Commit Standards**:
   - Use conventional commits (feat, fix, refactor, test, docs, chore)
   - No "Generated with Claude Code" text
   - No co-author attribution

---

## <¨ Output Format

Provide updates at each step:

```
## = Step 1: Explore
Analyzing codebase...
Found relevant files:
- Backend: klask-rs/src/services/search.rs
- Frontend: klask-react/src/features/search/SearchPageV3.tsx

## =Ý Step 2: Plan
[Implementation plan markdown]

## =» Step 3: Code (Launching parallel agents)
Launching rust-backend-expert and react-frontend-expert in parallel...
[Agent execution status]

## >ê Step 4: Test (Launching parallel agents)
Launching test-specialist agents for backend and frontend...
[Test results]

## = Step 5: Debug
All tests passing 

## =A Step 6: Review
Launching code-reviewer...
[Review summary]

##  Step 7: Verify
Running final checks...
 Backend tests: PASSED
 Frontend tests: PASSED
 Linting: PASSED
 Build: PASSED

## =æ Step 8: Commit
Created commit: feat: add language filter to search

## =€ Step 9: Push & CI Check
Pushed to: feature/language-filter
CI Status:  All checks passed
```

---

## =« Common Pitfalls to Avoid

1. **Sequential vs Parallel**: Don't launch agents sequentially when they can run in parallel
2. **Incomplete Testing**: Don't skip writing tests for edge cases
3. **Premature Committing**: Don't commit until ALL tests pass
4. **Ignoring Review**: Don't skip code review step
5. **CI Blindness**: Don't assume local tests guarantee CI success - always check

---

## =¡ Tips

- If feature is complex, break it into smaller sub-features
- Use existing similar features as reference
- Keep changes focused and atomic
- Update documentation if needed
- Consider backward compatibility
- Test manually in addition to automated tests

---

*This workflow ensures high-quality, well-tested features are delivered efficiently using parallel agent execution.*
