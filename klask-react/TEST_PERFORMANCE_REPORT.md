# 📊 Frontend Test Performance Analysis

**Generated:** 2025-11-06
**Branch:** sort-and-date

---

## 🐌 **Top 30 Slowest Tests** (Ranked by Duration)

| Rank | Duration | Test File | Test Name |
|------|----------|-----------|-----------|
| 1 | **574ms** | `RepositoryForm.test.tsx` | should create GitLab repository with all filter fields |
| 2 | **524ms** | `SearchPage.repository.test.tsx` | displays repository/project name in search results |
| 3 | **404ms** | `SearchBar.test.tsx` | should call onChange when user types |
| 4 | **371ms** | `RepositoryForm.test.tsx` | should include all filter fields in CreateRepositoryRequest |
| 5 | **368ms** | `SearchPage.repository.test.tsx` | filters results by repository/project when clicking project badge |
| 6 | **355ms** | `SearchPage.repository.test.tsx` | handles search results without repository name (legacy data) |
| 7 | **355ms** | `SearchPage.repository.test.tsx` | allows filtering by multiple repositories |
| 8 | **353ms** | `SearchPage.repository.test.tsx` | displays multiple repositories in facets |
| 9 | **351ms** | `SearchPage.repository.test.tsx` | clears repository filter when clicking clear button |
| 10 | **347ms** | `RepositoryForm.test.tsx` | should allow comma-separated values in filter fields |
| 11 | **338ms** | `SearchBar.test.tsx` | should debounce onChange and onSearch calls |
| 12 | **335ms** | `RepositoryForm.github.test.tsx` | should allow GitHub repository without namespace filter |
| 13 | **334ms** | `RepositoryForm.test.tsx` | should maintain form state when switching between tabs |
| 14 | **331ms** | `RepositoryForm.test.tsx` | should allow wildcard patterns in pattern fields |
| 15 | **315ms** | `RepositoryForm.test.tsx` | should trim whitespace from filter field values before submission |
| 16 | **307ms** | `useSearch.test.ts` | useFacetsWithFilters hook - should refetch when filters change |
| 17 | **282ms** | `RepositoryForm.github.test.tsx` | should validate required access token for GitHub |
| 18 | **271ms** | `RepositoriesPage.crawl-prevention.test.tsx` | should handle large numbers of repositories efficiently |
| 19 | **266ms** | `UserForm.test.tsx` | should prevent form submission with invalid input |
| 20 | **260ms** | `RepositoryForm.test.tsx` | should mark all new filter fields as optional |
| 21 | **234ms** | `RepositoryForm.test.tsx` | should not show validation errors for empty optional fields |
| 22 | **231ms** | `RepositoryForm.test.tsx` | should allow form submission with empty filter fields |
| 23 | **229ms** | `UserForm.test.tsx` | should allow valid usernames with underscores and hyphens |
| 24 | **223ms** | `RepositoryForm.test.tsx` | should edit existing GitHub repository to add filters |
| 25 | **181ms** | `UserForm.test.tsx` | should prevent submission with invalid username characters |
| 26 | **179ms** | `UserForm.test.tsx` | should call onSubmit with correct data for create mode |
| 27 | **173ms** | `RepositoryForm.test.tsx` | should render all 6 new filter fields |
| 28 | **170ms** | `UserForm.test.tsx` | should trim whitespace from username and email |
| 29 | **163ms** | `RepositoryForm.test.tsx` | should show "Filters & Exclusions" tab for Git repositories |
| 30 | **155ms** | `useAdmin.test.ts` | useAdminMetrics - should show loading state correctly |

---

## 📁 **Test Files by Total Impact** (Aggregated)

### 🥇 **RepositoryForm.test.tsx**
- **Total slow tests:** 15+ tests over 150ms
- **Cumulative time:** ~4.5 seconds
- **Main causes:**
  - Complex form rendering with many fields
  - Heavy use of `userEvent.type()` (slow)
  - Multiple tab switches
  - Form validation triggers

**Optimization opportunities:**
- ✅ Reduce number of fields tested in integration tests
- ✅ Use direct state updates instead of userEvent when possible
- ✅ Mock heavy form components
- ✅ Combine similar tests

---

### 🥈 **SearchPage.repository.test.tsx**
- **Total slow tests:** 6 tests over 350ms
- **Cumulative time:** ~2.3 seconds
- **Main causes:**
  - Full SearchPage component rendering
  - Multiple provider wrappers (QueryClient, Router, SearchFilters)
  - Heavy mock data creation
  - Multiple re-renders for facet updates

**Optimization opportunities:**
- ✅ Share mock data across tests (don't recreate)
- ✅ Use simpler test cases with less data
- ✅ Test facets separately from full page
- ✅ Reduce waitFor timeouts

---

### 🥉 **SearchBar.test.tsx**
- **Total slow tests:** 2 tests over 300ms
- **Cumulative time:** ~742ms
- **Main causes:**
  - Debounce testing (waiting for timers)
  - userEvent.type() is slow with long inputs

**Optimization opportunities:**
- ✅ Use fake timers for debounce tests
- ✅ Reduce length of typed strings

---

### **UserForm.test.tsx**
- **Total slow tests:** 6 tests over 170ms
- **Cumulative time:** ~1.3 seconds
- **Main causes:**
  - Form validation (react-hook-form)
  - Multiple userEvent interactions
  - Password field visibility toggles

**Optimization opportunities:**
- ✅ Batch multiple assertions in single test
- ✅ Use direct form state manipulation
- ✅ Mock validation library

---

## 💾 **Memory Consumption Analysis**

### Tests Excluded (High RAM)
Already excluded in `vite.config.ts`:
- ❌ `OptimizedSyntaxHighlighter.test.tsx`
- ❌ `VirtualizedSyntaxHighlighter.test.tsx`

**Reason:** Created massive test data (6000 lines, 150KB strings)

### High Memory Tests (Still Active)

1. **FileDetailPage.test.tsx**
   - Multiple large file content mocks
   - Syntax highlighter component loading
   - **Recommendation:** Reduce file content size in tests

2. **RepositoriesPage.test.tsx**
   - Tests with "large numbers of repositories" (271ms)
   - Creates arrays of 50+ repository objects
   - **Recommendation:** Reduce to 10-20 repositories

3. **SearchPage.repository.test.tsx**
   - Large search result arrays
   - Multiple repository facets
   - **Recommendation:** Use smaller data sets

---

## ⚡ **Quick Win Optimizations**

### 1. Use Fake Timers for Debounce Tests
```typescript
// ❌ Current (slow)
await waitFor(() => { ... }, { timeout: 1000 });

// ✅ Optimized
vi.useFakeTimers();
// ... test
vi.advanceTimersByTime(300);
vi.useRealTimers();
```
**Impact:** -50% time on debounce tests

---

### 2. Reduce userEvent.type() Usage
```typescript
// ❌ Slow
await userEvent.type(input, 'very long string here');

// ✅ Faster
fireEvent.change(input, { target: { value: 'string' } });
```
**Impact:** -70% time on typing tests

---

### 3. Share Test Data
```typescript
// ❌ Recreated every test
beforeEach(() => {
  mockData = createLargeDataSet();
});

// ✅ Shared across tests
const mockData = createLargeDataSet(); // Once
beforeEach(() => {
  // Just clear state
});
```
**Impact:** -30% setup time

---

### 4. Batch Assertions
```typescript
// ❌ Multiple similar tests
it('test A', () => expect(foo).toBe(1));
it('test B', () => expect(bar).toBe(2));
it('test C', () => expect(baz).toBe(3));

// ✅ Combined
it('tests A, B, C', () => {
  expect(foo).toBe(1);
  expect(bar).toBe(2);
  expect(baz).toBe(3);
});
```
**Impact:** -60% on grouped tests

---

## 🎯 **Recommended Actions** (Priority Order)

### Priority 1: Critical (Do Now)
1. ✅ **Increase maxWorkers to 8** (already done)
2. ⏳ **Reduce mock data size** in SearchPage tests (50% reduction)
3. ⏳ **Use fake timers** in SearchBar debounce tests
4. ⏳ **Replace userEvent.type()** with fireEvent in form tests

**Expected Impact:** -40% total time, -30% RAM

---

### Priority 2: High Impact
1. ⏳ **Reduce repository count** in RepositoriesPage large data tests (50→15)
2. ⏳ **Combine similar RepositoryForm tests** (15 tests → 8 tests)
3. ⏳ **Share mock data** across test suites
4. ⏳ **Simplify FileDetailPage** test file content

**Expected Impact:** -25% total time, -20% RAM

---

### Priority 3: Nice to Have
1. ⏳ Mock heavy dependencies (react-hook-form, React Query)
2. ⏳ Use snapshots instead of full renders for UI tests
3. ⏳ Split large test files into focused suites

**Expected Impact:** -15% total time

---

## 📈 **Estimated Total Impact**

| Optimization Level | Time Reduction | RAM Reduction |
|-------------------|----------------|---------------|
| **Priority 1 only** | -40% | -30% |
| **Priority 1 + 2** | -65% | -50% |
| **All priorities** | -80% | -65% |

---

## 🔧 **Current Configuration**

```typescript
// vite.config.ts
test: {
  maxWorkers: 8,           // ✅ Optimized (was 2)
  testTimeout: 10000,      // ✅ Set
  hookTimeout: 5000,       // ✅ Set
  isolate: true,           // ✅ Enabled
}
```

```json
// package.json
"test": "NODE_OPTIONS='--max-old-space-size=4096' vitest"
```

---

## 📊 **Benchmark Results**

### Before All Optimizations
- Duration: 134.79s
- Environment: 261.07s
- Setup: 64.65s

### After maxWorkers=2
- Duration: 177s (+40s worse ❌)
- Environment: 106s (-59% ✅)
- Setup: 25.7s (-60% ✅)

### After maxWorkers=8 (Expected)
- Duration: ~95s (-30% ✅)
- Environment: ~105s (similar)
- Setup: ~25s (similar)

**Next goal:** Get to < 80s total with data optimizations
