# 🎯 Real-World Agent Usage Examples

Practical examples showing how to use Klask's AI agents for common development tasks.

---

## 🦀 Backend Development Examples

### Example 1: Add New Search Filter

**Task**: Add a "programming language" filter to search

**Single Agent Approach**:
```
Use the rust-backend-expert agent to add a programming language filter:

1. Add language field to the Tantivy schema in src/services/search.rs
2. Update the query parser to accept language parameter
3. Modify the search function to filter by language
4. Update the API endpoint to accept language in query params

The languages to support: Rust, Python, JavaScript, TypeScript, Go, Java
```

**Multi-Agent Approach**:
```
Add programming language filter to Klask - run these agents in parallel:

1. rust-backend-expert:
   - Add language detection during indexing
   - Add language field to Tantivy schema
   - Update search API to filter by language

2. react-frontend-expert:
   - Add language filter dropdown to SearchFilters
   - Update SearchFiltersContext to handle language
   - Add language badges to search results

3. test-specialist:
   - Write backend tests for language filtering
   - Write frontend tests for language selector
   - Write integration tests end-to-end
```

### Example 2: Optimize Slow Query

**Task**: Search is slow for repositories with >50k files

```
Use the rust-backend-expert agent to optimize search performance:

The search query in src/services/search.rs is taking 5+ seconds for large repositories.

Please:
1. Profile the current query execution
2. Identify bottlenecks (likely in file_path or content fields)
3. Suggest and implement optimizations:
   - Better indexing strategy
   - Query result caching
   - Pagination improvements
4. Benchmark before/after performance

Target: < 500ms for any query
```

### Example 3: Add API Endpoint

**Task**: Add endpoint to export search results as CSV

```
Use the rust-backend-expert agent to create a CSV export endpoint:

Add POST /api/search/export endpoint that:
1. Accepts same parameters as /api/search
2. Returns CSV file with columns: repository, file_path, file_name, score
3. Limits to 10,000 results
4. Includes proper headers (Content-Type, Content-Disposition)
5. Handles errors gracefully

Then use test-specialist agent to write tests for this endpoint.
```

---

## ⚛️ Frontend Development Examples

### Example 4: Responsive Layout

**Task**: Search results not responsive on mobile

```
Use the react-frontend-expert agent to fix responsive layout:

The SearchResults component (src/components/search/SearchResults.tsx) doesn't work well on mobile:
- Grid is too wide
- Filters sidebar overlaps content
- Buttons are too small

Please:
1. Make grid responsive (1 col mobile, 2 cols tablet, 3+ cols desktop)
2. Convert filters to slide-out drawer on mobile
3. Increase button sizes on touch devices
4. Test on viewport widths: 320px, 768px, 1024px, 1920px
```

### Example 5: Add Feature

**Task**: Add "save search" functionality

```
Add save search feature - use react-frontend-expert agent:

Create functionality to save search queries:

1. Add "Save Search" button to SearchBar
2. Create SavedSearches component showing saved searches
3. Store in localStorage (key: 'klask-saved-searches')
4. Allow deleting saved searches
5. Clicking saved search populates SearchBar

Requirements:
- Max 10 saved searches
- Store: { query, filters, timestamp, name }
- Validate search names (3-50 chars)
- Show toast on save/delete
```

### Example 6: Loading States

**Task**: Add loading skeletons to improve perceived performance

```
Use react-frontend-expert agent to add loading skeletons:

Replace loading spinners with content skeletons in:
1. SearchResults - show 9 skeleton cards
2. RepositoriesList - show 5 skeleton rows
3. UsersList - show 10 skeleton rows

Use the Skeleton component pattern (create if needed).
Match the layout of actual content for smooth transitions.
```

---

## 🧪 Testing Examples

### Example 7: Fix Failing Tests

**Task**: Tests broken after refactoring

```
Use test-specialist agent to fix failing tests:

After refactoring SearchFiltersContext, 15 tests are failing in:
- src/features/search/__tests__/SearchPage.test.tsx
- src/components/search/__tests__/SearchFilters.test.tsx

Please:
1. Identify what changed in SearchFiltersContext
2. Update test mocks to match new structure
3. Fix assertions that are now incorrect
4. Ensure all tests pass
5. Verify no test coverage was lost
```

### Example 8: Increase Coverage

**Task**: Get to 100% test coverage for critical component

```
Use test-specialist agent to achieve 100% coverage:

The SearchFiltersContext (src/contexts/SearchFiltersContext.tsx) has only 67% coverage.

Please write tests for:
1. Edge cases (empty filters, invalid values)
2. Error states (API failures)
3. Race conditions (rapid filter changes)
4. Integration scenarios (with React Query)

Use vitest coverage report to verify 100% coverage:
npm test -- --coverage src/contexts/SearchFiltersContext.tsx
```

### Example 9: Debug React Query Issue

**Task**: Hook returns stale data

```
Use test-specialist agent to debug React Query issue:

The useRepositories hook is returning stale data after mutations.

Symptoms:
- Create repository succeeds but list doesn't update
- Need to refresh page to see new repository

The hook is in src/hooks/useRepositories.ts
Tests are in src/hooks/__tests__/useRepositories.test.ts

Please:
1. Verify queryClient.invalidateQueries is called correctly
2. Check query keys match between query and invalidation
3. Add test that reproduces the issue
4. Fix the root cause
```

---

## 🚀 Deployment Examples

### Example 10: Deploy New Version

**Task**: Deploy v2.1.0 to staging

```
Use deployment-expert agent to deploy to staging:

Deploy Klask v2.1.0 to staging environment:

1. Build Docker images:
   - klask-backend:v2.1.0
   - klask-frontend:v2.1.0

2. Update Helm chart with new image tags

3. Deploy to staging cluster (--kubeconfig ~/.kube/test)

4. Verify health:
   - All pods running
   - Backend /health returns 200
   - Frontend loads correctly
   - Database connection working

5. Run smoke tests on staging

Rollback plan: Keep previous deployment ready
```

### Example 11: Scale for Load Test

**Task**: Prepare for load testing with 10k concurrent users

```
Use deployment-expert agent to scale for load test:

We need to load test Klask with 10k concurrent users.

Please:
1. Scale backend deployment to 10 replicas
2. Increase PostgreSQL connection pool
3. Set resource limits appropriately (CPU/memory)
4. Configure horizontal pod autoscaling
5. Set up monitoring dashboards

Then verify:
- All pods are healthy
- Connection pool is not maxed out
- Resource usage is reasonable

Target: Handle 10k users with < 500ms response time
```

### Example 12: Fix Production Issue

**Task**: Backend pods crashing in production

```
Use deployment-expert agent to debug production crash:

Backend pods are crashing with OOMKilled in production.

Please:
1. Check pod logs for crash info
2. Review resource limits and usage
3. Identify memory leak or spike cause
4. Suggest fix (increase limits vs fix code)
5. Implement solution
6. Monitor for stability

Context:
- Crashes started after v2.0.0 deployment
- Happens after ~4 hours uptime
- Memory usage steadily increases
```

---

## 👁️ Code Review Examples

### Example 13: Security Audit

**Task**: Review authentication for security issues

```
Use code-reviewer agent for security audit:

Please review the authentication module for security vulnerabilities:

Files:
- klask-rs/src/api/auth.rs
- klask-rs/src/middleware/auth.rs
- klask-react/src/contexts/AuthContext.tsx

Check for:
1. SQL injection vulnerabilities
2. JWT token security (expiration, signing, storage)
3. Password hashing (algorithm, salt)
4. Session management
5. CORS configuration
6. Rate limiting on login endpoint
7. Credential exposure in logs

Provide severity ratings (Critical/High/Medium/Low) for findings.
```

### Example 14: Performance Review

**Task**: Review before production release

```
Use code-reviewer agent for performance review:

Review these changes for performance issues before v2.0.0 release:

Files:
- klask-rs/src/services/search.rs (search algorithm changes)
- klask-react/src/features/search/SearchPage.tsx (UI updates)

Check for:
1. Database query efficiency (N+1 problems)
2. Unnecessary re-renders in React
3. Memory leaks (closures, subscriptions)
4. Bundle size increases
5. API response times
6. Caching opportunities

Benchmark critical paths and suggest optimizations.
```

### Example 15: Best Practices Check

**Task**: Review PR from new team member

```
Use code-reviewer agent to review PR #234:

First-time contributor submitted PR #234 adding user preferences.

Please review for:
1. Code style consistency with project
2. TypeScript usage (no any, proper types)
3. Error handling patterns
4. Test coverage
5. Documentation
6. Accessibility (ARIA labels)

Provide constructive feedback with code examples.
Be welcoming and educational.
```

---

## 🔄 Multi-Agent Workflows

### Example 16: Complete Feature

**Task**: Add "repository stars" feature end-to-end

```
Add complete repository stars feature with these agents in parallel:

1. rust-backend-expert:
   Database & API:
   - Create stars table (repo_id, user_id, created_at)
   - Add POST /api/repositories/:id/star endpoint
   - Add DELETE /api/repositories/:id/star endpoint
   - Add star_count to repository model
   - Add is_starred flag to search results

2. react-frontend-expert:
   UI & State:
   - Add star button to RepositoryCard component
   - Create useStarRepository hook (React Query mutation)
   - Add star count display with icon
   - Show starred state (filled/outline star)
   - Add optimistic updates
   - Handle errors with toast notifications

3. test-specialist:
   Tests:
   - Backend: test star/unstar endpoints
   - Backend: test concurrent star attempts (unique constraint)
   - Frontend: test star button interactions
   - Frontend: test optimistic updates
   - Integration: test end-to-end flow

4. code-reviewer:
   Quality:
   - Security: ensure users can only star once per repo
   - Performance: add index on (repo_id, user_id)
   - UX: loading states, error handling
   - Accessibility: keyboard navigation, screen readers

5. deployment-expert:
   Deploy:
   - Create database migration
   - Deploy migration to staging
   - Deploy new backend version
   - Deploy new frontend version
   - Verify feature works in staging
```

### Example 17: Refactoring

**Task**: Refactor search service for maintainability

```
Refactor search service with multiple agents:

1. rust-backend-expert:
   - Split monolithic search.rs into modules:
     - query_parser.rs
     - indexer.rs
     - facets.rs
     - search_service.rs
   - Extract common patterns
   - Add documentation

2. test-specialist:
   - Update tests for new structure
   - Ensure same coverage
   - Add integration tests

3. code-reviewer:
   - Review module boundaries
   - Check for circular dependencies
   - Verify error handling

Run these sequentially (refactor → test → review)
```

### Example 18: Bug Investigation

**Task**: Search returns wrong results for certain queries

```
Debug search bug with systematic agent approach:

1. test-specialist:
   - Create failing test that reproduces the bug
   - Identify exact conditions that trigger it
   - Document expected vs actual behavior

2. rust-backend-expert:
   - Debug the search query execution
   - Identify root cause (query parsing, indexing, facets?)
   - Implement fix
   - Verify test now passes

3. test-specialist:
   - Add edge case tests to prevent regression
   - Verify fix doesn't break other tests

4. code-reviewer:
   - Review the fix for correctness
   - Check for similar issues elsewhere
   - Suggest preventive measures

Run these sequentially
```

---

## 💡 Pro Tips

### Combining Agents Effectively

**For new features**: backend → frontend → tests → review
**For bug fixes**: test → fix → test → review
**For refactoring**: plan → refactor → test → review → deploy
**For deployments**: test → build → deploy → monitor

### Being Specific

❌ Bad: "Make the search better"
✅ Good: "Optimize search query performance for repositories with >10k files by adding proper indexes and caching"

❌ Bad: "Fix the tests"
✅ Good: "Fix the 13 failing tests in useRepositories.edge-cases.test.tsx by properly setting up React Query mocks"

❌ Bad: "Add a feature"
✅ Good: "Add repository bookmarks feature with API endpoints, UI components, local storage, and tests"

### Parallel vs Sequential

**Parallel** (independent tasks):
- Backend + Frontend + Tests (different files)
- Multiple refactoring in different modules
- Multiple deployments to different environments

**Sequential** (dependent tasks):
- Test → Fix → Test → Review
- Build → Deploy → Verify
- Refactor → Update Tests → Review

---

## 🎓 Learning Path

### Week 1: Single Agents
- Try each agent with simple tasks
- Understand agent specializations
- Review agent outputs

### Week 2: Multi-Agent
- Combine 2-3 agents
- Try parallel execution
- Measure speed improvements

### Week 3: Complex Workflows
- Full feature development
- End-to-end bug fixes
- Production deployments

### Week 4: Optimization
- Create custom agents
- Optimize workflows
- Share team best practices

---

Ready to accelerate your development? Start with any example above! 🚀
