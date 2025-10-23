# 🚀 Quick Start - Klask AI Agents

Get started with Klask's AI-powered development workflow in 5 minutes!

## ✅ Prerequisites

Make sure you have:
- Claude Code CLI installed
- Klask repository cloned
- Backend and frontend running (see main README.md)

## 🎯 Your First Agent Task

Let's use an agent to add a simple feature. Try this:

### Example 1: Add a New Search Filter (Backend)

```
Hey, I want to add a new filter for file extensions in the search.

Use the rust-backend-expert agent to:
1. Add a file_extension field to the search query
2. Update the Tantivy query parser to support it
3. Add the filter to the search service

Then use the test-specialist agent to write tests for this new filter.
```

### Example 2: Fix UI Component (Frontend)

```
The search results grid is not responsive on mobile.

Use the react-frontend-expert agent to make the SearchResults component responsive using TailwindCSS breakpoints.
```

### Example 3: Fix Failing Tests

```
There are failing tests in useRepositories.edge-cases.test.tsx.

Use the test-specialist agent to:
1. Identify why the tests are failing
2. Fix the root cause (not just symptoms)
3. Ensure all tests pass
```

## 🔄 Testing the Workflow

### Step 1: Make a Change
Edit any file in `klask-rs/` or `klask-react/`:

```bash
# Example: Edit a Rust file
nano klask-rs/src/services/search.rs
```

### Step 2: Automatic Checks
The post-code-change hook will automatically run relevant tests!

### Step 3: Commit with Quality Checks
```bash
git add .
git commit -m "Add file extension filter"
# Pre-commit hook runs automatically:
# ✅ Formats code
# ✅ Runs linting
# ✅ Runs tests
```

## 🤝 Multi-Agent Workflows

For complex features, use multiple agents in parallel:

### Full Feature Example: "Star Repository"

```
I want to add a "star repository" feature to Klask.

Please use these agents in parallel:

1. rust-backend-expert:
   - Add stars table to PostgreSQL
   - Create API endpoints (POST /api/repositories/:id/star, DELETE /api/repositories/:id/star)
   - Add star count to repository model

2. react-frontend-expert:
   - Add star button to RepositoryCard component
   - Create useStarRepository hook with React Query
   - Add star count display
   - Update UI to show starred state

3. test-specialist:
   - Write backend tests for star endpoints
   - Write frontend tests for star button
   - Write integration tests

4. code-reviewer:
   - Review security (ensure users can only star once)
   - Review performance (proper indexing on stars table)
   - Review UI/UX (loading states, error handling)
```

## 🧪 Test the Agents Individually

### Test rust-backend-expert
```
Use the rust-backend-expert agent to analyze the current search performance and suggest optimizations for repositories with > 10,000 files.
```

### Test react-frontend-expert
```
Use the react-frontend-expert agent to add loading skeletons to the search results while data is being fetched.
```

### Test test-specialist
```
Use the test-specialist agent to achieve 100% test coverage for the SearchFiltersContext component.
```

### Test deployment-expert
```
Use the deployment-expert agent to create a Helm values file for deploying Klask to a staging environment.
```

### Test code-reviewer
```
Use the code-reviewer agent to review the authentication module in klask-rs/src/api/auth.rs for security vulnerabilities.
```

## 📊 Monitoring Agent Performance

### Check What Agents Are Doing
Agents will report their progress:
- What files they're reading
- What changes they're making
- What tests they're running
- What issues they find

### Verify Results
Always review agent work:
```bash
# Check backend changes
cd klask-rs && cargo test

# Check frontend changes
cd klask-react && npm test

# Check code quality
./.claude/hooks/pre-commit.sh
```

## 🎓 Best Practices

### DO ✅
- Be specific in your requests
- Use the right agent for the job
- Run agents in parallel for independent tasks
- Review agent changes before committing
- Use test-specialist after code changes
- Use code-reviewer for security-sensitive code

### DON'T ❌
- Use generic requests (be specific!)
- Skip testing after agent changes
- Commit without running pre-commit hook
- Ignore agent suggestions without understanding why

## 🔧 Troubleshooting

### Agent Not Working?
```bash
# Check agent files exist
ls -la .claude/agents/

# Check agent file format
cat .claude/agents/rust-backend-expert.md
```

### Hooks Not Running?
```bash
# Make hooks executable
chmod +x .claude/hooks/*.sh

# Test hook manually
./.claude/hooks/pre-commit.sh
```

### Tests Failing After Agent Changes?
```
Use the test-specialist agent to debug and fix the failing tests systematically.
```

## 🚀 Next Steps

1. **Try the examples above** to get familiar with agents
2. **Combine agents** for complex features
3. **Customize agents** in `.claude/agents/` for your specific needs
4. **Share your workflows** with the team

## 💡 Pro Tips

### Speed Up Development
- Use agents in **parallel** (specify "run in parallel")
- Be **specific** about file paths and requirements
- Provide **context** about what you've already tried

### Better Results
- Let agents **read code first** before modifying
- Use **test-specialist** to verify changes
- Use **code-reviewer** as final check

### Avoid Common Mistakes
- Don't say "fix the bug" - describe the bug specifically
- Don't say "make it better" - define "better"
- Don't skip the review step - always verify agent work

## 📚 Resources

- [Full Agent Documentation](./.claude/README.md)
- [Klask Project Guide](../README.md)
- [Claude Agent SDK Docs](https://docs.claude.com/en/api/agent-sdk/overview)

Ready to supercharge your Klask development! 🎉
