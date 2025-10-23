# Master Rules
- Do NEVER add "Generated with Claude Code" on pull request
- Do NEVER add co-author on the commit you create
- on commit message, use like any citizen
- Always use `gh cli` to get information about issues / pull request etc..
- Always work in "klask-rs" directory for the new backend rewrite in Rust and "klask-react" for the new frontend in React
- Always use the workflow describe in command /explore-plan-code-test
- execute in background tasks the frontend and backend

## 🤖 AI Agents System
Klask uses specialized AI agents for accelerated development. See `.claude/README.md` for full documentation.

### Available Agents
- **rust-backend-expert**: Tantivy search, Axum API, PostgreSQL - use for backend development
- **react-frontend-expert**: React, TypeScript, React Query, TailwindCSS - use for frontend development
- **test-specialist**: Test writing and debugging for Rust and React - use when tests fail
- **deployment-expert**: Kubernetes, Docker, CI/CD - use for infrastructure and deployment
- **code-reviewer**: Security, performance, best practices - use after significant code changes

### Using Agents
Simply mention the agent in your request:
```
Use the rust-backend-expert agent to add a language filter to the search service
```

For complex tasks, use multiple agents in parallel:
```
Add bookmark feature: rust-backend-expert for API, react-frontend-expert for UI, test-specialist for tests - run in parallel
```

### Automatic Quality Checks
- Pre-commit hook runs tests and linting automatically
- Post-code-change hook verifies modifications
- Located in `.claude/hooks/`

## how to run backend
```
cd klask-rs && cargo run --bin klask-rs
```

## how to run frontend
```
cd klask-react && npm run dev
```

## the database
the database postgreSQl is already running in a docker container, open on the port 5432

## pour les tests avec kubernetes
pour la commande kubectl, il faut renseigner le fichier kubeconfig suivant :
`--kubeconfig ~/.kube/test`
