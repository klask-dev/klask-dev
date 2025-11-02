# Architecture Diagram: /search vs /facets

## System Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Frontend (React/TypeScript)                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  SearchPageV3.tsx                  SearchFiltersContext.tsx         │
│  ┌──────────────────┐              ┌──────────────────────────┐    │
│  │ Search Page      │              │ Filter State Management  │    │
│  │                  │              │                          │    │
│  │ useMultiSelect   │              │ - staticFilters          │    │
│  │ Search() ──────┐ │              │ - filterFacets           │    │
│  │                │ │              │ - searchResultsFacets    │    │
│  └──────────────────┘              │ - merged display         │    │
│                                    │                          │    │
│                                    │ useFacetsWithFilters()   │    │
│                                    │ useSearchFilters()       │    │
│                                    └──────────────────────────┘    │
│                                                                      │
│  React Query Hooks Layer:                                           │
│  ┌──────────────────────┐  ┌──────────────────────┐  ┌──────────┐ │
│  │ useMultiSelectSearch │  │ useFacetsWithFilters │  │ useSearch│ │
│  │                      │  │                      │  │ Filters  │ │
│  │ - Query required     │  │ - Query optional     │  │ - Static │ │
│  │ - Debounce: none     │  │ - Debounce: 500ms    │  │ - Cache: │ │
│  │ - Cache: per query   │  │ - Cache: per filter  │  │   5 min  │ │
│  └──────────────────────┘  └──────────────────────┘  └──────────┘ │
│           ↓                         ↓                      ↓        │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │                   API Client (fetch layer)                 │   │
│  │                  /lib/api.ts + hooks direct calls         │   │
│  └────────────────────────────────────────────────────────────┘   │
│           ↓                         ↓                               │
└─────────────┼─────────────────────────┼───────────────────────────┘
              │                         │
              ↓                         ↓
     ┌─────────────────┐       ┌─────────────────┐
     │   /search       │       │   /facets       │
     └─────────────────┘       └─────────────────┘
              ↓                         ↓
┌─────────────────────────────────────────────────────────────────────┐
│                    Backend (Rust/Axum)                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  /api/search.rs::search_files()    /api/search.rs::get_facets_      │
│  ┌───────────────────────┐         ┌───────────────────────┐       │
│  │ SearchRequest         │         │ FacetsRequest         │       │
│  │ ├─ q: String          │         │ ├─ query: String      │       │
│  │ ├─ limit: u32         │         │ ├─ (optional)         │       │
│  │ ├─ page: u32          │         │ └─ filters...         │       │
│  │ ├─ filters...         │         │                       │       │
│  │ └─include_facets      │         │ Purpose: Facets only  │       │
│  │                       │         │ Sets internally:      │       │
│  │ Purpose: Search +     │         │ ├─ limit = 0          │       │
│  │ optional facets       │         │ ├─ include_facets=true│       │
│  │                       │         │ └─ query = "*" if     │       │
│  │                       │         │   query is empty      │       │
│  └───────────────────────┘         └───────────────────────┘       │
│           ↓                               ↓                         │
│  ┌─────────────────────────────────────────────────────────┐       │
│  │         SearchService::search(SearchQuery)              │       │
│  │                                                         │       │
│  │  1. Parse query parameters                             │       │
│  │  2. Build Tantivy query with filters                   │       │
│  │  3. Execute search                                     │       │
│  │  4. Fetch results (if limit > 0)                       │       │
│  │  5. Compute facets (if include_facets = true)          │       │
│  │                                                         │       │
│  │     ↓ ↓ ↓                                              │       │
│  │  collect_facets_from_search_results()                  │       │
│  │  ├─ Uses Tantivy aggregations                          │       │
│  │  ├─ Excludes self-dimension filters                    │       │
│  │  │  (e.g., repo facets exclude repo filter)            │       │
│  │  └─ Returns SearchFacets struct                        │       │
│  │                                                         │       │
│  └─────────────────────────────────────────────────────────┘       │
│           ↓                         ↓                               │
│  SearchResponse {         SearchFacets {                            │
│    results: [Results],      repositories: [...],                   │
│    total: u64,              projects: [...],                       │
│    page: u32,               versions: [...],                       │
│    limit: u32,              extensions: [...],                     │
│    facets: SearchFacets     size_ranges: [...]                     │
│  }                          }                                       │
│           ↓                         ↓                               │
└─────────────┼───────────────────────┼──────────────────────────────┘
              ↓                       ↓
       (Results +            (Facets only,
        optional              no results)
        facets)
```

## Data Flow for Common Scenarios

### Scenario 1: Page Load (Initial Filter Display)

```
                                                  Time: t=0
                                                  
SearchPageV3 mounts
       │
       ├─→ SearchFiltersContext provides initial state
       │
       └─→ useSearchFilters() 
           │   (No parameters)
           │
           └─→ /facets
               │
               └─→ Backend: AllQuery across entire index
                   │
                   └─→ Returns facets for all documents
                       (5-minute cached)
                       │
                       └─→ Populates filter panel with counts
                           "Projects: 500 total"
                           "Versions: 30 total"
```

### Scenario 2: User Changes Filters (No Active Search)

```
                                                  Time: t=5000
                                                  
User clicks "python" filter checkbox
       │
       ├─→ SearchFiltersContext.setFilters({project: ['python']})
       │
       ├─→ useFacetsWithFilters detected filter change
       │   (Debounced 500ms)
       │
       └─→ /facets?projects=python
           │
           └─→ Backend: AllQuery + project:python filter
               │
               └─→ Aggregation: What other options exist with this filter?
                   │
                   └─→ Returns facets for filtered set
                       "Versions: 8 available in python"
                       "Extensions: 150 .py, 20 .pyx"
                       │
                       └─→ Update filter panel counts dynamically
```

### Scenario 3: User Searches with Filters

```
                                                  Time: t=8000
                                                  
User types "async" in search box
       │
       ├─→ useMultiSelectSearch() triggered
       │   (Query required, enabled: !!query.trim())
       │
       └─→ /search?q=async&projects=python&include_facets=true
           │
           └─→ Backend: text:"async" + project:python filter
               │
               ├─→ TopDocs: Retrieve matching documents (limit=20)
               │   │
               │   └─→ 47 documents match
               │
               ├─→ Aggregation: Facets within search results
               │   │
               │   └─→ Returns facets for "async" in "python"
               │       "Versions in async results: 3.6 (15), 3.9 (32)"
               │       "Extensions in async results: .py (44), .pyx (3)"
               │
               └─→ UI: Show results + updated facet counts
                   Emphasizes: "These counts are for 'async' query"
```

### Scenario 4: User Adds Another Filter During Search

```
                                                  Time: t=12000
                                                  
User clicks "3.9" version filter while viewing "async" results
       │
       ├─→ SearchFiltersContext.setFilters({
       │   project: ['python'],
       │   version: ['3.9']
       │   })
       │
       ├─→ useFacetsWithFilters detected new filter
       │   (Debounced 500ms)
       │
       └─→ /facets?query=async&projects=python&versions=3.9
           │
           └─→ Backend: text:"async" + project:python + version:3.9
               │
               └─→ Aggregation: Facets for this combined filter set
                   │
                   └─→ Returns facets within this context
                       "Repositories in async/python/3.9: main (20), legacy (5)"
                       "Extensions in async/python/3.9: .py (32), .pyx (0)"
                       │
                       └─→ Context updates with these counts
```

### Scenario 5: User Clears Search

```
                                                  Time: t=15000
                                                  
User clears "async" search box
       │
       ├─→ useMultiSelectSearch() disabled
       │   (enabled: !!query.trim() is now false)
       │
       ├─→ useFacetsWithFilters still active
       │   (filters still: {project: ['python'], version: ['3.9']})
       │
       └─→ /facets?projects=python&versions=3.9
           │
           └─→ Backend: AllQuery + project:python + version:3.9
               │
               └─→ Aggregation: All documents with these filters
                   │
                   └─→ Returns global facets for filter set
                       "Repositories in python/3.9: main (150), legacy (80)"
                       "Extensions in python/3.9: .py (500), .pyx (50)"
                       │
                       └─→ UI: Show filter panel with these counts
                           (No search results displayed)
```

## Query Decision Tree

```
                          User Action
                              │
                    ┌─────────┴─────────┐
                    │                   │
            Performs Search    Changes Filters Only
                    │                   │
                    ↓                   ↓
        Does query already exist?   Are filters active?
                    │                   │
          ┌─────────┴─────────┐        YES ─→ useFacetsWithFilters()
          │                   │             └─→ /facets
         YES                  NO             └─→ Dynamic counts
          │                   │
          ↓                   ↓         NO → useSearchFilters()
    useMultiSelectSearch()  (First search?)  └─→ /facets (cached)
    ├─ /search with          │               └─→ Static counts
    │  include_facets=true   └─ YES: Fetch
    │                           │
    │ Returns:              NO:  └─ Search disabled
    │ ├─ Results                │
    │ ├─ Facets (context)        └─ useFacetsWithFilters still active
    │ └─ Pagination info            │
    │                            └─ /facets with filters
    │
    ├─ SearchFiltersContext
    │  updates with facets
    │
    └─ UI: Results + updated counts
```

## Facet Computation Flow

```
collect_facets_from_search_results()
│
├─ For REPOSITORIES facet:
│  ├─ Build query with: project + version + extension filters (NO repo filter)
│  ├─ Execute aggregation on "repository" field
│  └─ Return: [("main", 150), ("legacy", 80), ...]
│
├─ For PROJECTS facet:
│  ├─ Build query with: repository + version + extension filters (NO project filter)
│  ├─ Execute aggregation on "project" field
│  └─ Return: [("python", 500), ("node", 300), ...]
│
├─ For VERSIONS facet:
│  ├─ Build query with: repository + project + extension filters (NO version filter)
│  ├─ Execute aggregation on "version" field
│  └─ Return: [("3.9", 600), ("3.8", 150), ...]
│
├─ For EXTENSIONS facet:
│  ├─ Build query with: repository + project + version filters (NO extension filter)
│  ├─ Execute aggregation on "extension" field
│  └─ Return: [(".py", 850), (".pyx", 100), ...]
│
└─ For SIZE_RANGES facet:
   ├─ Build query with: all other filters (NO size filter)
   ├─ Execute aggregation on size buckets
   └─ Return: [("1-10KB", 200), ("10-100KB", 300), ...]

Result: Facets that answer "What else is available?"
        Not "What matches these filters?" (That would be redundant)
```

## Cache Strategy

```
useSearchFilters()
├─ Query Key: ['search', 'static-filters']
├─ Stale Time: 5 minutes (expensive query, stable data)
├─ Calls: Once per page load
└─ Data: All documents aggregate

useFacetsWithFilters()
├─ Query Key: ['search', 'facets', filterKey, query]
├─ Stale Time: 60 seconds
├─ Debounce: 500ms (smooth UX)
├─ Placeholder: keepPreviousData (no flicker)
└─ Data: Filtered aggregate

useMultiSelectSearch()
├─ Query Key: ['search', 'multiselect', query, filters, currentPage]
├─ Stale Time: 30 seconds
├─ Debounce: None
├─ Retry: 3 attempts
└─ Data: Results + context facets

Independent caching allows:
- Static filters cached separately from dynamic filters
- Search results cached per page/filters/query combo
- Optimal expiry times per data freshness needs
```

## Why NOT Single Endpoint?

### Combined Request Example (NOT RECOMMENDED)
```
/search?q=&facets_only=true&projects=python&versions=3.9

Problems:
1. Still ambiguous: empty query means AllQuery or no-op?
2. Forced to handle both result and no-result cases
3. UI can't distinguish "static" vs "filtered" vs "search" facets
4. Can't optimize queries independently
5. Caching becomes complex (multiple response types per key)
6. Would break existing /search API contract

Solution: Keep separate endpoints ✓
```

