# 🔧 Klask Development Guide for Claude Code

## 🎯 Master Rules
- **NEVER** add "Generated with Claude Code" on pull requests
- **NEVER** add co-author on commits you create
- Use commit messages like any regular contributor
- Always use `gh cli` to get information about issues/pull requests
- Work in `klask-rs/` for backend (Rust) and `klask-react/` for frontend (React)
- **Always** use the `/explore-plan-code-test` workflow for features
- Execute backend and frontend in background when testing

---

## 📂 Project Structure

```
klask-dev/
├── klask-rs/          # Rust backend (Axum + Tantivy + PostgreSQL)
├── klask-react/       # React frontend (TypeScript + TailwindCSS)
├── .claude/           # AI agents, commands, hooks
│   ├── agents/        # Specialized AI agents
│   ├── commands/      # Custom slash commands
│   └── hooks/         # Pre-commit, post-code-change hooks
└── CLAUDE.md         # This file
```

---

## 🗄️ Database Management

### PostgreSQL Container
The PostgreSQL database runs in a Docker container named `klask-postgres-dev`.

#### Start the database
```bash
docker compose -f docker-compose.dev.yml up -d
```

#### Stop the database
```bash
docker stop klask-postgres-dev
```

#### Access PostgreSQL CLI
```bash
docker exec -it klask-postgres-dev psql -U klask_user -d klask_dev
```

#### Common PostgreSQL commands
```sql
-- List all tables
\dt

-- Describe table schema
\d repositories
\d files

-- View repositories
SELECT id, name, url, repository_type FROM repositories;

-- Count indexed files
SELECT COUNT(*) FROM files;

-- Check crawl status
SELECT name, last_crawled, last_crawl_duration_seconds FROM repositories;

-- Exit psql
\q
```

#### Connection Details
- **Host**: localhost
- **Port**: 5432
- **Database**: klask
- **User**: klask
- **Password**: klask
- **Connection String**: `postgresql://klask:klask@localhost:5432/klask`

#### Backup/Restore
```bash
# Backup
docker exec klask-postgres-dev pg_dump -U klask klask > backup.sql

# Restore
docker exec -i klask-postgres-dev psql -U klask_user klask < backup.sql
```

---

## 🦀 Backend (Rust)

### Directory Structure
```
klask-rs/
├── src/
│   ├── api/              # API endpoints (Axum routes)
│   ├── models/           # Database models
│   ├── repositories/     # Database queries (repository pattern)
│   ├── services/         # Business logic
│   │   ├── crawler/      # Modular crawler (Git, GitLab, GitHub)
│   │   ├── search.rs     # Tantivy search engine
│   │   ├── encryption.rs # Token encryption
│   │   └── ...
│   ├── bin/              # Binary executables
│   └── main.rs           # Server entry point
├── tests/                # Integration tests
├── migrations/           # SQL migrations
└── Cargo.toml            # Dependencies
```

### Start Backend
```bash
cd klask-rs
cargo run --bin klask-rs
```

The backend will start on **http://localhost:3000**

### Run in Background (for testing)
```bash
cd klask-rs
cargo run --bin klask-rs > /tmp/klask-backend.log 2>&1 &
echo $! > /tmp/klask-backend.pid
```

### Stop Background Backend
```bash
kill $(cat /tmp/klask-backend.pid)
rm /tmp/klask-backend.pid
```

### Development Commands
```bash
# Run tests
cargo test

# Run specific test
cargo test test_name

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Build
cargo build

# Build release
cargo build --release

# Check without building
cargo check
```

### Database Migrations
```bash
# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Create new migration
sqlx migrate add migration_name
```

### Key Files to Know
- `src/services/crawler/service.rs` - Main crawler orchestration
- `src/services/search.rs` - Tantivy search implementation
- `src/api/repositories.rs` - Repository CRUD endpoints
- `src/api/search.rs` - Search API endpoints

---

## ⚛️ Frontend (React)

### Directory Structure
```
klask-react/
├── src/
│   ├── api/              # API client (React Query)
│   ├── components/       # Reusable components
│   ├── features/         # Feature-specific modules
│   │   ├── search/       # Search feature
│   │   ├── repositories/ # Repository management
│   │   └── ...
│   ├── hooks/            # Custom React hooks
│   ├── types/            # TypeScript types
│   └── App.tsx           # Main app component
├── public/               # Static assets
└── package.json          # Dependencies
```

### Start Frontend
```bash
cd klask-react
npm run dev
```

The frontend will start on **http://localhost:5173**

### Run in Background (for testing)
```bash
cd klask-react
npm run dev > /tmp/klask-frontend.log 2>&1 &
echo $! > /tmp/klask-frontend.pid
```

### Stop Background Frontend
```bash
kill $(cat /tmp/klask-frontend.pid)
rm /tmp/klask-frontend.pid
```

### Development Commands
```bash
# Install dependencies
npm install

# Run tests
npm test

# Run specific test
npm test -- SearchPage

# Lint
npm run lint

# Fix linting issues
npm run lint:fix

# Type check
npm run type-check

# Build
npm run build

# Preview production build
npm run preview
```

### Key Files to Know
- `src/features/search/SearchPageV3.tsx` - Main search interface
- `src/api/repositories.ts` - Repository API hooks
- `src/api/search.ts` - Search API hooks
- `src/hooks/useSearch.ts` - Search state management

---

## 🤖 AI Agents System

Klask uses specialized AI agents for accelerated development. See `.claude/README.md` for full documentation.

### Available Agents

#### 1. **rust-backend-expert** 🦀
Expert in Rust backend development for Klask.

**When to use:**
- Adding/modifying API endpoints
- Optimizing Tantivy search queries
- Database schema changes
- Crawler improvements
- Performance optimization

**Example:**
```
Use rust-backend-expert to add a language filter to the search API
```

#### 2. **react-frontend-expert** ⚛️
Expert in React/TypeScript frontend.

**When to use:**
- Creating/modifying React components
- Adding UI features
- Styling with TailwindCSS
- State management with React Query
- TypeScript type issues

**Example:**
```
Use react-frontend-expert to add a dark mode toggle to the settings page
```

#### 3. **test-specialist** 🧪
Expert in writing and debugging tests.

**When to use:**
- Tests are failing
- Need to write comprehensive test suite
- Debugging test issues
- Improving test coverage

**Example:**
```
Use test-specialist to fix failing tests in crawler_test.rs
```

#### 4. **deployment-expert** 🚀
Expert in Kubernetes, Docker, CI/CD.

**When to use:**
- Deploying to Kubernetes
- Docker configuration issues
- CI/CD pipeline problems
- Infrastructure management

**Example:**
```
Use deployment-expert to deploy the latest version to test environment
```

#### 5. **code-reviewer** 👁️
Expert code reviewer for security, performance, best practices.

**When to use:**
- After completing a significant code change
- Before creating a pull request
- Security audit
- Performance review

**Example:**
```
Use code-reviewer to review the new authentication module
```

#### 6. **debug-specialist** 🔍
Expert in debugging and troubleshooting.

**When to use automatically (proactive):**
- Tests fail
- Compilation errors
- Runtime errors
- Unexpected behavior

**Example:**
```
Use debug-specialist to investigate why the crawler is timing out on large repositories
```

### Agent Triggers

Agents are **triggered automatically** in these scenarios:

1. **test-specialist** - When `cargo test` or `npm test` fails
2. **debug-specialist** - When build fails or runtime errors occur
3. **code-reviewer** - After significant code changes (automatically by post-code-change hook)

You can also **manually invoke** agents by mentioning them in your request.

---

## 🔄 Development Workflow

### The `/explore-plan-code-test` Command

This is the **primary workflow** for implementing new features. It orchestrates multiple agents in parallel for maximum efficiency.

#### What it does:
1. **Explore** - Analyze codebase to understand existing patterns
2. **Plan** - Create implementation plan
3. **Code** - Write code using specialized agents:
   - `rust-backend-expert` for API/backend
   - `react-frontend-expert` for UI/frontend
   - Both run **in parallel**
4. **Test** - Write tests using `test-specialist`:
   - Backend tests (Rust)
   - Frontend tests (React)
   - Both run **in parallel**
5. **Debug** - `debug-specialist` fixes any failing tests
6. **Review** - `code-reviewer` audits the changes
7. **Verify** - Ensure all tests pass
8. **Commit** - Create commit and push
9. **CI Check** - Verify GitHub Actions CI passes

#### Usage:
```
/explore-plan-code-test Add bookmark feature with star icon in search results
```

#### Example Flow:
```
User: /explore-plan-code-test Add language filter to search

1. [Explore] Analyzing search.rs and SearchPageV3.tsx...
2. [Plan] Implementation plan:
   - Backend: Add language field to search query
   - Frontend: Add language filter dropdown
   - Tests: Unit + integration tests
3. [Code - Parallel]
   - rust-backend-expert: Adding language filter to SearchService...
   - react-frontend-expert: Adding language dropdown to SearchFilters...
4. [Test - Parallel]
   - test-specialist: Writing backend tests...
   - test-specialist: Writing frontend tests...
5. [Debug] All tests pass ✅
6. [Review] code-reviewer: Code quality check...
7. [Commit] git commit -m "feat: add language filter to search"
8. [Push] git push origin feature/language-filter
9. [CI] Checking GitHub Actions... ✅ All checks passed
```

### The `/devops` Command

This command handles **all DevOps operations** - deployment, monitoring, troubleshooting, and infrastructure management.

#### What it does:
1. **Deploy** - Deploy Klask to Kubernetes (test/production)
2. **Status** - Check health and status of all components
3. **Logs** - View and analyze application logs
4. **Troubleshoot** - Debug deployment and runtime issues
5. **Build** - Build and push Docker images
6. **CI/CD** - Check and manage GitHub Actions pipelines
7. **Database** - Manage PostgreSQL in Kubernetes
8. **Rollback** - Revert to previous deployment

#### Usage Examples:
```bash
# Deploy to test environment
/devops deploy test

# Check application status
/devops status

# View backend logs with errors
/devops logs backend --errors --tail 100

# Troubleshoot failing pod
/devops troubleshoot pod klask-backend-xxx

# Build and push images
/devops build all --push --tag v2.3.0

# Check CI/CD pipeline
/devops ci status

# Database backup
/devops database backup

# Rollback deployment
/devops rollback
```

See `.claude/commands/devops.md` for complete documentation.

---

## 🔗 GitHub Integration

### Check PR Status
```bash
# List open PRs
gh pr list

# View PR details
gh pr view <pr-number>

# Check PR CI status
gh pr checks <pr-number>

# View PR diff
gh pr diff <pr-number>
```

### Check CI Status
```bash
# View workflow runs
gh run list

# View specific run
gh run view <run-id>

# Watch run in real-time
gh run watch
```

### Create PR
```bash
# Create PR from current branch
gh pr create --title "feat: add language filter" --body "Description here"

# Create PR with labels
gh pr create --title "fix: search bug" --label bug,priority-high
```

---

## 🧪 Testing Strategy

### Backend Tests (Rust)
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test file
cargo test --test crawler_test

# Run tests in specific module
cargo test services::search

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Frontend Tests (React)
```bash
# Run all tests
npm test

# Run tests in watch mode
npm test -- --watch

# Run specific test file
npm test SearchPage

# Run with coverage
npm test -- --coverage
```

### Integration Tests
```bash
# Backend integration tests
cd klask-rs
cargo test --test '*'

# Frontend integration tests
cd klask-react
npm test -- --testPathPattern="integration"
```

---

## 🚀 Kubernetes Testing

### Kubectl Configuration
For Kubernetes commands, use the test kubeconfig:
```bash
kubectl --kubeconfig ~/.kube/test get pods
kubectl --kubeconfig ~/.kube/test logs <pod-name>
kubectl --kubeconfig ~/.kube/test describe pod <pod-name>
```

---

## 🐛 Debugging Tips

### Backend Debugging
```bash
# Enable debug logs
RUST_LOG=debug cargo run --bin klask-rs

# Enable trace logs (very verbose)
RUST_LOG=trace cargo run --bin klask-rs

# Debug specific module
RUST_LOG=klask_rs::services::search=debug cargo run --bin klask-rs
```

### Frontend Debugging
```bash
# React DevTools
# Install: npm install -g react-devtools

# Enable source maps in production
npm run build -- --sourcemap
```

### Database Debugging
```bash
# Watch queries in real-time
docker exec -it klask-postgres-dev psql -U klask_user -d klask_dev -c "ALTER SYSTEM SET log_statement = 'all';"
docker restart klask-postgres-dev
docker logs -f klask-postgres-dev
```

---

## 📝 Commit Message Format

Follow conventional commits:
```
feat: add language filter to search
fix: resolve search pagination bug
refactor: split monolithic crawler into modules
test: add integration tests for GitLab crawler
docs: update API documentation
chore: upgrade dependencies
```

---

## 🔒 Security Notes

- Never commit `.env` files with real credentials
- Use encryption service for storing tokens in database
- OAuth tokens are passed via `http.extraHeader` (not in URLs)
- Database passwords should be rotated regularly
- Review all code with `code-reviewer` agent before merging

---

## 📚 Additional Resources

- **Backend API docs**: http://localhost:3000/api-docs (when running)
- **Tantivy docs**: https://docs.rs/tantivy/latest/tantivy/
- **Axum docs**: https://docs.rs/axum/latest/axum/
- **React Query docs**: https://tanstack.com/query/latest
- **Agents documentation**: `.claude/README.md`
- **Examples**: `.claude/EXAMPLES.md`

---

## 🎨 Code Style

### Rust
- Use `cargo fmt` for formatting
- Follow Rust idioms (no `unwrap()` in production)
- Prefer `?` operator for error propagation
- Use meaningful variable names
- Add doc comments for public APIs

### TypeScript/React
- Use `npm run lint:fix` for formatting
- Prefer functional components with hooks
- Use TypeScript strictly (no `any`)
- Extract reusable logic into custom hooks
- Keep components small and focused

---

## 🏁 Quick Start Checklist

Before starting work:
- [ ] Database is running: `docker start klask-postgres-dev`
- [ ] Backend is running: `cd klask-rs && cargo run --bin klask-rs`
- [ ] Frontend is running: `cd klask-react && npm run dev`
- [ ] Latest code: `git pull origin master`
- [ ] Dependencies updated: `cargo update` and `npm install`

For new features:
- [ ] Use `/explore-plan-code-test` command
- [ ] Let agents work in parallel
- [ ] Review code with `code-reviewer`
- [ ] Ensure all tests pass
- [ ] Check CI on GitHub after pushing

---

*Happy coding! 🚀*
