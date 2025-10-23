# 🤖 Klask AI Agent System

This directory contains specialized AI agents, hooks, and commands to accelerate Klask development.

## 📁 Directory Structure

```
.claude/
├── agents/           # Specialized AI agents
├── hooks/            # Automation hooks
├── commands/         # Custom slash commands (future)
└── README.md         # This file
```

## 🎯 Available Agents

### 1. **rust-backend-expert** 🦀
Expert in Rust backend development for Klask search engine.

**Use for:**
- Tantivy search optimization
- Axum API development
- PostgreSQL integration
- Performance optimization
- Backend bug fixes

**Example usage:**
```
Use the rust-backend-expert agent to optimize the search query performance for repositories with many files.
```

### 2. **react-frontend-expert** ⚛️
Expert in React/TypeScript frontend development.

**Use for:**
- React component development
- TypeScript type safety
- React Query state management
- TailwindCSS styling
- Frontend bug fixes

**Example usage:**
```
Use the react-frontend-expert agent to add a new language filter to the search UI.
```

### 3. **test-specialist** 🧪
Expert in writing and fixing tests for both Rust and React.

**Use for:**
- Fixing failing tests
- Writing comprehensive test suites
- Debugging test issues
- Test coverage improvements

**Example usage:**
```
Use the test-specialist agent to fix all failing tests in useRepositories.edge-cases.test.tsx
```

### 4. **deployment-expert** 🚀
Expert in Kubernetes, Docker, and CI/CD.

**Use for:**
- Deploying Klask to Kubernetes
- Docker configuration
- CI/CD pipeline issues
- Infrastructure management

**Example usage:**
```
Use the deployment-expert agent to deploy the latest version to the test environment.
```

### 5. **code-reviewer** 👁️
Expert code reviewer focused on security, performance, and best practices.

**Use for:**
- Security audits
- Performance reviews
- Code quality checks
- Best practices enforcement

**Example usage:**
```
Use the code-reviewer agent to review the new authentication module for security issues.
```

## 🪝 Available Hooks

### pre-commit.sh
Runs automatically before each git commit to ensure code quality.

**What it does:**
- ✅ Formats Rust code with `cargo fmt`
- ✅ Runs `cargo clippy` with strict warnings
- ✅ Runs all Rust tests
- ✅ Fixes ESLint issues automatically
- ✅ Runs all React tests
- ✅ Type checks TypeScript code

**Manual execution:**
```bash
./.claude/hooks/pre-commit.sh
```

### post-code-change.sh
Runs automatically after code modifications to verify changes.

**What it does:**
- 🔄 Detects which part of the codebase changed
- 🧪 Runs relevant tests for changed modules
- ⚡ Quick feedback on code changes

**Manual execution:**
```bash
./.claude/hooks/post-code-change.sh klask-rs/src/services/search.rs
```

## 🚀 Quick Start Guide

### Using Agents in Conversation

Simply mention the agent you want to use in your request:

```
Hey, use the rust-backend-expert agent to add a new filter for programming languages in the search service.
```

Or be more explicit:

```
I need help with the frontend. Can the react-frontend-expert agent help me create a responsive search results grid?
```

### Using Multiple Agents in Parallel

For complex tasks, use multiple agents concurrently:

```
I want to add a "star repository" feature:
1. Use rust-backend-expert to create the API endpoint
2. Use react-frontend-expert to add the UI button
3. Use test-specialist to write tests for both
4. Use code-reviewer to review the implementation

Please run these agents in parallel to speed up the work.
```

### Automatic Hook Execution

Hooks run automatically, but you can also trigger them manually:

```bash
# Before committing manually check everything
./.claude/hooks/pre-commit.sh

# After making changes, verify tests
./.claude/hooks/post-code-change.sh klask-react/src/components/SearchBar.tsx
```

## 🎓 Best Practices

### When to Use Which Agent

| Task | Agent | Why |
|------|-------|-----|
| Add search filter backend | rust-backend-expert | Knows Tantivy + PostgreSQL |
| Add search filter frontend | react-frontend-expert | Knows React Query + contexts |
| Tests failing | test-specialist | Expert at debugging tests |
| Deploy to Kubernetes | deployment-expert | Knows infrastructure |
| Security review | code-reviewer | Security-focused analysis |

### Combining Agents Effectively

**Full Feature Development:**
```
Add "save search" feature:
1. rust-backend-expert: API + database
2. react-frontend-expert: UI components
3. test-specialist: Comprehensive tests
4. code-reviewer: Security + performance review
```

**Bug Fix Workflow:**
```
Fix search pagination bug:
1. test-specialist: Reproduce bug with test
2. rust-backend-expert: Fix the bug
3. test-specialist: Verify fix with tests
4. code-reviewer: Review the fix
```

**Deployment Workflow:**
```
Deploy to production:
1. test-specialist: Ensure all tests pass
2. code-reviewer: Final security review
3. deployment-expert: Deploy + monitor
```

## 📊 Agent Performance Tips

### Faster Development
- Use agents in **parallel** for independent tasks
- Be **specific** in your requests
- Provide **context** about what you've already tried

### Better Results
- Let agents read code **before** modifying
- Trust agent expertise in their domain
- Combine agents for **end-to-end** features

## 🔧 Customization

### Adding New Agents

Create a new markdown file in `.claude/agents/`:

```markdown
---
name: your-agent-name
description: When to use this agent
---

# Your Agent Name

Agent instructions and expertise...
```

### Modifying Hooks

Edit files in `.claude/hooks/` to customize automation behavior.

### Creating Commands

(Future feature) Custom slash commands will go in `.claude/commands/`

## 🆘 Troubleshooting

### Agent Not Responding
- Check agent file exists in `.claude/agents/`
- Verify YAML frontmatter is correct
- Try restarting Claude Code

### Hooks Not Running
- Ensure hooks are executable: `chmod +x .claude/hooks/*.sh`
- Check hook output for errors
- Verify paths in hook scripts

### Tests Failing After Agent Changes
- Use test-specialist agent to debug
- Check that dependencies are installed
- Verify database is running (for backend)

## 📚 Additional Resources

- [Claude Agent SDK Documentation](https://docs.claude.com/en/api/agent-sdk/overview)
- [Klask Project Documentation](../README.md)
- [Contributing Guidelines](../CONTRIBUTING.md)

## 🎯 Next Steps

1. **Try the agents**: Start with simple tasks to understand each agent
2. **Combine agents**: Use multiple agents for complex features
3. **Customize**: Adapt agents to your workflow
4. **Automate**: Set up Git hooks for automatic checks

Happy coding with AI assistance! 🚀
