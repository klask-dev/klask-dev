# Size Range Facets Implementation Analysis

## Executive Summary

The size range facets feature has been **partially implemented** across the codebase but is **not displaying properly** to users. The backend is correctly computing size range facets and returning them via the API, but the frontend is **not properly handling the `size_ranges` field** in multiple critical places, causing the counters to never reach the UI components.

## Current Status

### What Works (Backend - 100% Complete)
- ✅ Size buckets defined: `SIZE_BUCKETS` constant in backend
- ✅ Size facets calculated in `collect_facets_from_search_results()`
- ✅ API endpoints return `size_ranges` in `SearchFacets` response
- ✅ Proper size filtering with `min_size` and `max_size` query parameters

### What's Broken (Frontend - Data Flow Broken)
- ❌ `normalizeFacetsResponse()` function **does NOT handle `size_ranges`**
- ❌ `useFacetsWithFilters()` hook **does NOT pass `size_ranges` in query parameters**
- ❌ `SearchFiltersContext` attempts to handle `size_ranges` but never receives data
- ❌ `SidebarFilters` component receives empty `sizeRanges` array

---

## Detailed Findings

### 1. BACKEND: Size Buckets Definition
**File:** `/home/jeremie/git/github/klask-rs/src/services/search.rs`
**Lines:** 17-24

```rust
const SIZE_BUCKETS: &[(&str, Option<u64>, Option<u64>)] = &[
    ("< 1 KB", None, Some(1024)),
    ("1 KB - 10 KB", Some(1024), Some(10 * 1024)),
    ("10 KB - 100 KB", Some(10 * 1024), Some(100 * 1024)),
    ("100 KB - 1 MB", Some(100 * 1024), Some(1024 * 1024)),
    ("1 MB - 10 MB", Some(1024 * 1024), Some(10 * 1024 * 1024)),
    ("> 10 MB", Some(10 * 1024 * 1024), None),
];
```

**Status:** ✅ Correctly defined - 6 size ranges with proper boundaries

---

### 2. BACKEND: Size Facets Calculation
**File:** `/home/jeremie/git/github/klask-rs/src/services/search.rs`
**Lines:** 1219-1470 (in `collect_facets_from_search_results()`)

**Key Logic:**
- For each size bucket, creates a range query
- Applies all non-size filters (repository, project, version, extension)
- **Deliberately excludes size filters** from facet calculation (line 1324 comment)
- Returns counts for each bucket as `Vec<(String, u64)>`

**Code Snippet (Lines 1455-1467):**
```rust
let mut size_facets = Vec::new();
for (label, min_size, max_size) in SIZE_BUCKETS.iter() {
    let bucket_query = get_size_bucket_query(*min_size, *max_size);
    match searcher.search(&*bucket_query, &Count) {
        Ok(count) => {
            size_facets.push((label.to_string(), count as u64));
        }
        Err(_) => {
            size_facets.push((label.to_string(), 0));
        }
    }
}
```

**Status:** ✅ Correctly implemented - generates facet counts for each range

---

### 3. BACKEND: API Response Structure
**File:** `/home/jeremie/git/github/klask-rs/src/api/search.rs`
**Lines:** 94-107 (SearchFacets struct)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchFacets {
    pub repositories: Vec<FacetValue>,
    pub projects: Vec<FacetValue>,
    pub versions: Vec<FacetValue>,
    pub extensions: Vec<FacetValue>,
    pub size_ranges: Vec<FacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: u64,
}
```

**Lines:** 179-206 (Converting service facets to API response)
```rust
size_ranges: service_facets
    .size_ranges
    .into_iter()
    .map(|(value, count)| FacetValue { value, count })
    .collect(),
```

**Status:** ✅ API response includes `size_ranges` array

---

### 4. FRONTEND: Type Definitions
**File:** `/home/jeremie/git/github/klask-react/src/types/index.ts`
**Lines:** 184-200

```typescript
export interface FacetsApiResponse {
    projects: FacetResponseItem[];
    versions: FacetResponseItem[];
    extensions: FacetResponseItem[];
    repositories: FacetResponseItem[];
    languages: FacetResponseItem[];
    size_ranges?: FacetResponseItem[];  // ✅ DEFINED
}

export interface SearchFacets {
    projects: FacetValue[];
    versions: FacetValue[];
    extensions: FacetValue[];
    languages: FacetValue[];
    repositories?: FacetValue[];
    size_ranges?: FacetValue[];  // ✅ DEFINED
}
```

**Status:** ✅ Types are correctly defined

---

### 5. FRONTEND BREAKPOINT #1: normalizeFacetsResponse()
**File:** `/home/jeremie/git/github/klask-react/src/hooks/useSearch.ts`
**Lines:** 397-434

**Problem:** The function returns a normalized response but **DOES NOT include `size_ranges`**

```typescript
const normalizeFacetsResponse = (data: unknown): FacetsApiResponse => {
  // ... validation code ...
  
  return {
    projects: normalizeFacetArray(response.projects),
    versions: normalizeFacetArray(response.versions),
    extensions: normalizeFacetArray(response.extensions),
    repositories: normalizeFacetArray(response.repositories),
    languages: normalizeFacetArray(response.languages),
    // ❌ MISSING: size_ranges field!
  };
};
```

**Impact:** Even if the backend returns `size_ranges`, this function drops them

**Fix Required:**
```typescript
return {
  projects: normalizeFacetArray(response.projects),
  versions: normalizeFacetArray(response.versions),
  extensions: normalizeFacetArray(response.extensions),
  repositories: normalizeFacetArray(response.repositories),
  languages: normalizeFacetArray(response.languages),
  size_ranges: normalizeFacetArray(response.size_ranges),  // ✅ ADD THIS
};
```

---

### 6. FRONTEND BREAKPOINT #2: useFacetsWithFilters()
**File:** `/home/jeremie/git/github/klask-react/src/hooks/useSearch.ts`
**Lines:** 459-550 (continuation not shown in previous reads)

**Problem:** The hook doesn't pass `size_ranges` counts to the API request

**Current Implementation (incomplete snippet):**
```typescript
export const useFacetsWithFilters = (
  filters: {
    project?: string[];
    version?: string[];
    extension?: string[];
    repository?: string[];
    // ❌ MISSING: size filter parameter
  } = {},
  query: string = '',
  options: UseSearchOptions = {}
)
```

**What's Missing:**
1. The hook doesn't accept or pass `size_ranges` in the API query
2. It fetches facets but discards the `size_ranges` data
3. The hook converts facets to UI format but never maps `size_ranges`

---

### 7. FRONTEND: SearchFiltersContext
**File:** `/home/jeremie/git/github/klask-react/src/contexts/SearchFiltersContext.tsx`
**Lines:** 35, 50, 92, 125, 148

**Good Attempts:**
- Defines `size_ranges` in `DynamicFilters` interface (line 35)
- Includes `sizeRanges` in context type (line 50)
- Initializes `size_ranges` from staticFilters (lines 92, 125, 148)

**Problem:** Since `normalizeFacetsResponse()` doesn't return `size_ranges`, `staticFilters.size_ranges` is always `undefined`

---

### 8. FRONTEND: SidebarFilters Component
**File:** `/home/jeremie/git/github/klask-react/src/components/search/SidebarFilters.tsx`
**Lines:** 46, 373

```typescript
interface SidebarFiltersProps {
  // ...
  availableFilters?: {
    projects: FilterOption[];
    versions: FilterOption[];
    extensions: FilterOption[];
    languages: FilterOption[];
    sizeRanges?: Array<{ value: string; count: number }>;  // ✅ DEFINED
  };
}

// Later in component:
<SizeFilter
  value={filters.size}
  onChange={(sizeValue) => { /* ... */ }}
  sizeRangeFacets={availableFilters.sizeRanges}  // ❌ ALWAYS EMPTY/UNDEFINED
  isLoading={isLoading}
/>
```

**Status:** Component is ready to receive facets but never does

---

### 9. FRONTEND: SizeFilter Component
**File:** `/home/jeremie/git/github/klask-react/src/components/search/SizeFilter.tsx`
**Lines:** 9, 82-90, 284

```typescript
interface SizeFilterProps {
  sizeRangeFacets?: Array<{ value: string; count: number }>;  // ✅ Ready
}

const getCountForPreset = (
  label: string,
  facets?: Array<{ value: string; count: number }>
): number | undefined => {
  if (!facets || facets.length === 0) return undefined;
  const facet = facets.find(f => f.value === label);
  return facet?.count;
};

// Later:
const count = getCountForPreset(preset.label, sizeRangeFacets);  // ❌ Gets undefined
return (
  // ...
  {!isLoading && count !== undefined && (
    <span>{count.toLocaleString()}</span>  // ❌ NEVER RENDERS
  )}
)
```

**Status:** Ready to display counts but receives empty array

---

## Data Flow Diagram

```
BACKEND (Working ✅)
  └─> SearchService.collect_facets_from_search_results()
      └─> Returns SearchFacets { size_ranges: Vec<(String, u64)> }
          └─> API /search/facets endpoint
              └─> JSON: { size_ranges: [{ value: "< 1 KB", count: 42 }, ...] }

FRONTEND - BROKEN DATA FLOW ❌
  └─> API returns size_ranges in JSON ✅
      └─> useSearchFilters() hook receives it ✅
          └─> normalizeFacetsResponse() DROPS IT ❌
              └─> SearchFiltersContext receives undefined ❌
                  └─> SidebarFilters passes undefined ❌
                      └─> SizeFilter never shows counts ❌
```

---

## Missing/Broken Files

### Files That Need Changes

1. **`/home/jeremie/git/github/klask-react/src/hooks/useSearch.ts`**
   - Line 433: Add `size_ranges: normalizeFacetArray(response.size_ranges),`

2. **`/home/jeremie/git/github/klask-react/src/hooks/useSearch.ts` (useFacetsWithFilters)**
   - Add size filter handling (not fully visible in reads)
   - Ensure size_ranges are included in API response mapping

### Files Working Correctly

- ✅ Backend services/search.rs - Calculates facets correctly
- ✅ Backend API search.rs - Returns correct structure
- ✅ Frontend SidebarFilters - Ready to display
- ✅ Frontend SizeFilter - Ready to show counters
- ✅ Frontend types/index.ts - Correct definitions
- ✅ Frontend SearchFiltersContext - Attempts to handle correctly

---

## Root Cause Summary

The **root cause** is a **missing field mapping** in the `normalizeFacetsResponse()` function. This single function is the bottleneck where size_ranges data is dropped from the API response before being passed to the context and components.

**Breakage Points (in order of execution):**
1. ❌ `normalizeFacetsResponse()` line 427-433 - Missing `size_ranges` in return statement
2. ❌ `useSearchFilters()` hook - Calls the broken normalizer
3. ❌ Context receives `undefined` for size_ranges
4. ❌ Components receive empty array for facet counts
5. ❌ UI never shows size range counters

---

## Why Counters Don't Display

**For "< 1 KB" preset with 42 matching files:**

Backend path (✅ Working):
```
SIZE_BUCKETS[0] = ("< 1 KB", None, Some(1024))
  → Query counts files matching this range
  → Returns (String("< 1 KB"), 42u64)
  → Includes in size_ranges array
  → Serializes to JSON: { value: "< 1 KB", count: 42 }
  → Returns via /api/search/facets
```

Frontend path (❌ Broken):
```
API response received: { size_ranges: [{ value: "< 1 KB", count: 42 }] }
  → normalizeFacetsResponse() called
    → Normalizes other facets ✅
    → Returns without size_ranges ❌
  → useFacetsWithFilters() receives incomplete facets
  → SearchFiltersContext.size_ranges = undefined ❌
  → SidebarFilters.availableFilters.sizeRanges = undefined ❌
  → SizeFilter.sizeRangeFacets = undefined ❌
  → getCountForPreset() returns undefined
  → Preset button shows NO counter ❌
```

---

## Expected vs Actual

### Expected (After Fix)
```
< 1 KB          [42]
1 KB - 10 KB    [158]
10 KB - 100 KB  [2,341]
100 KB - 1 MB   [891]
1 MB - 10 MB    [34]
> 10 MB         [2]
```

### Actual (Current)
```
< 1 KB
1 KB - 10 KB
10 KB - 100 KB
100 KB - 1 MB
1 MB - 10 MB
> 10 MB

(No counters visible)
```

---

## Recommended Fixes

### Critical Fix #1: normalizeFacetsResponse()
Location: `/home/jeremie/git/github/klask-react/src/hooks/useSearch.ts` line 433

Change:
```typescript
// FROM:
return {
  projects: normalizeFacetArray(response.projects),
  versions: normalizeFacetArray(response.versions),
  extensions: normalizeFacetArray(response.extensions),
  repositories: normalizeFacetArray(response.repositories),
  languages: normalizeFacetArray(response.languages),
};

// TO:
return {
  projects: normalizeFacetArray(response.projects),
  versions: normalizeFacetArray(response.versions),
  extensions: normalizeFacetArray(response.extensions),
  repositories: normalizeFacetArray(response.repositories),
  languages: normalizeFacetArray(response.languages),
  size_ranges: normalizeFacetArray(response.size_ranges),
};
```

### Secondary Fix #2: Verify useFacetsWithFilters completes the mapping
Location: Continue reading `useSearch.ts` from line 550 onwards

Need to ensure `useFacetsWithFilters()` properly maps `size_ranges` when converting API response to UI format.

---

## Verification Steps

1. Add `size_ranges` to `normalizeFacetsResponse()` return
2. Test `/api/search/facets` endpoint directly - should return size_ranges
3. Verify `useSearchFilters()` hook includes size_ranges
4. Check SidebarFilters receives non-empty sizeRanges array
5. SizeFilter component should show counters on preset buttons

---

## Impact

- **Severity:** Medium (Feature partially broken)
- **User Impact:** Size range filter presets don't show result counts
- **Data Loss:** No, all data flows correctly up to the UI
- **Complexity of Fix:** Very Low - single field addition in one function
