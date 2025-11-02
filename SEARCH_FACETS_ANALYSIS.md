# Comprehensive Analysis: /search vs /facets API Endpoints in Klask

## Executive Summary

The `/search` and `/facets` endpoints are **not redundant** but serve **different architectural purposes** in the Klask codebase. While both compute facet data identically, they are called in different scenarios for distinct user experience requirements.

---

## 1. Backend API Endpoints Analysis

### 1.1 `/search` Endpoint
**File**: `/home/jeremie/git/github/klask-dev/klask-rs/src/api/search.rs:130-217`

**Request Structure** (`SearchRequest`):
```rust
pub struct SearchRequest {
    pub q: Option<String>,                          // Search query (required)
    pub query: Option<String>,                      // Alternative query parameter
    pub limit: Option<u32>,                         // Results per page (max 1000)
    pub page: Option<u32>,                          // Page number for pagination
    pub repositories: Option<String>,               // Comma-separated filter values
    pub projects: Option<String>,
    pub versions: Option<String>,
    pub extensions: Option<String>,
    pub min_size: Option<u64>,                      // Size range filters (bytes)
    pub max_size: Option<u64>,
    pub include_facets: Option<bool>,               // Optional facets in response
}
```

**Response Structure** (`SearchResponse`):
```rust
pub struct SearchResponse {
    pub results: Vec<SearchResult>,                 // Actual search results
    pub total: u64,                                 // Total matching documents
    pub page: u32,                                  // Current page
    pub limit: u32,                                 // Results per page
    pub facets: Option<SearchFacets>,              // Optional facets field
}

pub struct SearchFacets {
    pub repositories: Vec<FacetValue>,
    pub projects: Vec<FacetValue>,
    pub versions: Vec<FacetValue>,
    pub extensions: Vec<FacetValue>,
    pub size_ranges: Vec<FacetValue>,
}
```

**Key Characteristics**:
- **Required**: Search query (q or query)
- **Optional**: Pagination, filters, facets inclusion
- **Returns**: Search results + optional facets
- **Facets Logic** (lines 180-206): Facets computed when include_facets=true
- **Filter Validation**: NO validation in /search endpoint

### 1.2 `/facets` Endpoint
**File**: `/home/jeremie/git/github/klask-dev/klask-rs/src/api/search.rs:219-317`

**Request Structure** (`FacetsRequest`):
```rust
pub struct FacetsRequest {
    pub query: Option<String>,                      // Optional search query
    pub repositories: Option<String>,               // Comma-separated filter values
    pub projects: Option<String>,
    pub versions: Option<String>,
    pub extensions: Option<String>,
    pub min_size: Option<u64>,                      // Size range filters
    pub max_size: Option<u64>,
}
```

**Response Structure**: Direct SearchFacets response (no results, no pagination)

**Key Characteristics**:
- **Required**: None (can call with no parameters)
- **Optional**: Filters, search query
- **Returns**: ONLY facets (no search results)
- **Facets Logic** (lines 257-268): limit=0, include_facets=true, uses "*" if no query
- **Filter Validation**: YES (strict validation of all parameters)

---

## 2. Frontend API Client & Hooks

**File**: `/home/jeremie/git/github/klask-react/src/hooks/useSearch.ts`

### Hook Comparison

| Hook | Endpoint | Purpose | Query Required | Returns |
|------|----------|---------|-----------------|---------|
| `useMultiSelectSearch()` | `/search` | Search with results + facets | YES | Results + facets |
| `useFacetsWithFilters()` | `/facets` | Filter panel updates | NO | Facets only |
| `useSearchFilters()` | `/facets` | Static filters (all docs) | NO | Facets only |

#### useMultiSelectSearch (lines 130-211)
- Called from SearchPageV3.tsx (line 102)
- Always sets `include_facets: 'true'` (line 189)
- Returns both results and query-specific facets
- Used for active search with results display

#### useFacetsWithFilters (lines 461-572)
- Called from SearchFiltersContext.tsx (line 112)
- Debounced 300-500ms to avoid excessive requests
- Only called when filters are active (line 555)
- Uses `keepPreviousData` for smooth UX
- Returns only facets for updating filter panel counters

#### useSearchFilters (lines 109-127)
- Called from SearchFiltersContext.tsx (line 73)
- Static filter list (all documents)
- Cached for 5 minutes
- Called once on component mount

---

## 3. Facet Computation (Identical Logic)

**File**: `/home/jeremie/git/github/klask-rs/src/services/search.rs:955-1087`

Both endpoints use identical `collect_facets_from_search_results()` method.

**Key Insight** (lines 1089-1093):
```
Each facet shows counts for available options when OTHER dimensions are filtered:
- Repository facets: apply project, version & extension filters (NOT repository)
- Project facets: apply repository, version & extension filters (NOT project)
- Version facets: apply repository, project & extension filters (NOT version)
- Extension facets: apply repository, project & version filters (NOT extension)
```

This is standard faceted navigation logic. Both endpoints compute facets identically.

---

## 4. Data Flow Architecture

### SearchFiltersContext (SearchFiltersContext.tsx:1-150+)

Three independent facet sources:

1. **staticFilters** (useSearchFilters) 
   - All available filters across entire index
   - Cached 5 minutes
   - Used as fallback

2. **filterFacets** (useFacetsWithFilters)
   - Counts based on selected filters
   - Debounced 500ms
   - Updates when user changes filters

3. **searchResultsFacets** (from useMultiSelectSearch)
   - Counts from search results
   - Query-specific
   - Updates when search is performed

**Merging Strategy** (lines 134-144):
```typescript
if (filterFacets) {
  setLastValidFacets({
    ...filterFacets,
    size_ranges: lastValidFacets?.size_ranges || filterFacets.size_ranges,
  });
}
```

---

## 5. Detailed Usage Scenarios

### Scenario A: Search Without Filter Changes
```
User types "react"
  ↓
useMultiSelectSearch() → /search?q=react&include_facets=true
  ↓
Response: {results: [...], facets: {...}}
  ↓
updateDynamicFilters(facets) updates SearchFiltersContext
  ↓
UI: Results on right, updated filter counters on left
```
**API Calls**: 1x /search

### Scenario B: Filter Change Without Active Search  
```
User selects filter "python" project
  ↓
useFacetsWithFilters() triggered (debounced 500ms)
  ↓
/facets?projects=python
  ↓
Response: {projects: [...], versions: [...], ...}
  ↓
setLastValidFacets(filterFacets) updates context
  ↓
UI: Filter panel counters updated
```
**API Calls**: 1x /facets

### Scenario C: Search + Then Filter
```
Step 1: useMultiSelectSearch() → /search?q=react&include_facets=true
        Results + facets shown

Step 2: User selects filter → useFacetsWithFilters()
        /facets?query=react&projects=python
        Counts adjusted for selected filters within search query

Step 3: User clears search → useMultiSelectSearch() disabled
        /facets?projects=python still active
        Shows counts for selected filters only
```
**API Calls**: Multiple /search and /facets as needed

---

## 6. Are Facets Identical?

**Identical Structure**: YES
Both return `SearchFacets` with same fields and types.

**Identical Values**: NO
Values depend on context:

| Endpoint | Parameters | Example Response | Meaning |
|----------|-----------|------------------|---------|
| `/search?q=react&include_facets=true` | Query search | `projects: [("my-proj", 150)]` | 150 docs with "react" in my-proj |
| `/facets` | (none) | `projects: [("my-proj", 5000)]` | 5000 total docs in my-proj |
| `/facets?query=react&projects=my-proj` | Query + filters | `versions: [("1.0", 80), ("2.0", 70)]` | Available versions in filtered set |

---

## 7. Why Both Endpoints? Architectural Justification

### Performance
- `/facets` with `limit=0` is much faster than `/search` with results
- Users change filters frequently without performing searches
- Separating prevents unnecessary result fetching

### Caching Strategy
- Static filters: 5-minute cache
- Dynamic filters: per-filter-state cache  
- Search results: per-query/filters/page cache
- Separate endpoints enable independent cache strategies

### UX Requirements
- Filter panel needs global counts on page load
- Search results need query-specific counts
- Mixing these would require fetching results just for counters

### API Design
- `/search`: Document retrieval with optional context
- `/facets`: Aggregation computation without document overhead
- Clear separation of concerns
- Independent evolution

### Query Optimization
- `/facets` with no query uses `AllQuery` (optimize for scan)
- `/search` always has text search component (optimize for matching)
- Different execution strategies

---

## 8. Is There Redundancy?

### Actual Redundancy: NO
- Different primary purposes
- Different response types
- Different call patterns
- Different backend computation

### Perceived Redundancy: MINIMAL
- Both compute facets identically (intentional for consistency)
- This ensures facet calculations are consistent across endpoints

### Design Rationale for Identical Facet Logic
Reusing the same facet computation ensures:
1. Consistency between endpoints
2. Single source of truth for facet logic
3. Easier maintenance
4. No divergence in calculations

---

## 9. Consolidation Feasibility

### Could They Be Combined?

**Technical Answer**: Yes, but not recommended.

**Why Not**:
1. **Breaking Change**: Forces all clients to always fetch results (slower)
2. **UX Regression**: Filter panel would need 50 results just for counts
3. **Performance Loss**: Every filter change would fetch results
4. **Forced Coupling**: Can't evolve endpoints independently
5. **Cache Complexity**: Single entry can't satisfy different expiry needs

### If Consolidation Were Required

**Option 1: Enhanced /search**
```
/search?q=react&response_type=facets_only
Response: facets for "react" query without results
Benefits: Existing /search users unaffected
```

**Option 2: Smart Query Parameter**
```
/search?q=react&include_results=false
Response: Same as /facets but via /search
Would replace /facets endpoint
```

**Option 3: Keep Current Design** (RECOMMENDED)
- Two focused endpoints
- Each optimized for its use case
- Clear separation of concerns
- Independent caching and evolution

---

## 10. Summary Code References

| Component | File | Lines | Purpose |
|-----------|------|-------|---------|
| /search endpoint | klask-rs/src/api/search.rs | 130-217 | Search with optional facets |
| /facets endpoint | klask-rs/src/api/search.rs | 219-317 | Facets with optional query |
| SearchRequest/Response | klask-rs/src/api/search.rs | 53-108 | Request/response structures |
| FacetsRequest | klask-rs/src/api/search.rs | 72-83 | Facets request structure |
| collect_facets_from_search_results | klask-rs/src/services/search.rs | 955-1087 | Facet computation logic |
| useMultiSelectSearch | klask-react/src/hooks/useSearch.ts | 130-211 | Search hook with results |
| useFacetsWithFilters | klask-react/src/hooks/useSearch.ts | 461-572 | Filter facets hook |
| useSearchFilters | klask-react/src/hooks/useSearch.ts | 109-127 | Static filters hook |
| SearchFiltersContext | klask-react/src/contexts/SearchFiltersContext.tsx | 1-150+ | Filter state management |
| SearchPageV3 usage | klask-react/src/features/search/SearchPageV3.tsx | 102-127 | Search page implementation |
| API client search | klask-react/src/lib/api.ts | 273-289 | API client method |

---

## Conclusion

The `/search` and `/facets` endpoints represent **well-architected separation of concerns**:

- **`/search`** optimized for document retrieval with optional context-specific facets
- **`/facets`** optimized for facet aggregation without document overhead
- Both use identical facet computation (ensures consistency)
- Each has independent caching and performance optimization
- Separation enables efficient UI patterns

**The current architecture is sound. No consolidation is recommended.**

