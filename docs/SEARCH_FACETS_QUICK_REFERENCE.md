# Quick Reference: /search vs /facets API Endpoints

## One-Page Comparison

### Endpoint Purposes
| Aspect | /search | /facets |
|--------|---------|---------|
| **Primary Purpose** | Retrieve documents matching query | Return filter aggregations |
| **Requires Query** | YES (must provide search text) | NO (optional) |
| **Returns Results** | YES (paginated, 1-50 documents) | NO (limit=0 internally) |
| **Returns Facets** | Only if `include_facets=true` | Always |
| **Common Use Case** | When user enters search query | When user changes filters |
| **Performance** | Slower (fetches results + facets) | Faster (facets only) |
| **Filter Validation** | NO | YES (strict) |

### Frontend Hooks
| Hook | Calls | Purpose | When Used |
|------|-------|---------|-----------|
| `useMultiSelectSearch()` | `/search` | Search with results | Search page (SearchPageV3.tsx:102) |
| `useFacetsWithFilters()` | `/facets` | Filter count updates | Context (SearchFiltersContext.tsx:112) |
| `useSearchFilters()` | `/facets` | Static filter list | Context (SearchFiltersContext.tsx:73) |

### Facet Data
- **Structure**: Identical in both (repositories, projects, versions, extensions, size_ranges)
- **Values**: Different based on context
  - `/search?q=react`: counts for "react" results only
  - `/facets`: counts for all documents
  - `/facets?projects=python`: counts with python project filter applied

### Key Decision Point
**Choose /facets when**: 
- Updating filter panel (user changed filters)
- Showing global available options
- Need fast aggregation without results

**Choose /search with include_facets=true when**:
- User performed a search
- Need both results AND context-specific facet counts
- Showing search results page

### Facet Collection Logic
Both use identical logic from `collect_facets_from_search_results()`:
- Each facet excludes its own dimension filter
- Example: "Repository" facets exclude repository filter but include project/version/extension filters
- This enables faceted navigation: "What other projects have results?"

### Code References
- **Backend**: `/klask-rs/src/api/search.rs` (lines 130-317)
- **Facet Logic**: `/klask-rs/src/services/search.rs` (lines 955-1087)
- **Frontend Hooks**: `/klask-react/src/hooks/useSearch.ts` (multiple sections)
- **Context**: `/klask-react/src/contexts/SearchFiltersContext.tsx` (lines 1-150+)

### Are They Redundant?
**NO**. They serve different purposes:
1. Different response types (results vs facets-only)
2. Different call patterns (query-driven vs filter-driven)
3. Different caching strategies (per-query vs per-filter-state)
4. Different performance characteristics (result-fetching vs aggregation-only)

The shared facet logic is intentional (consistency), not redundancy.

### Should They Be Combined?
**NO (not recommended)**. Reasons:
1. Would force fetching results just to get filter counts
2. Would slow down filter panel updates
3. Would couple search and filters
4. Can't have independent cache expiry
5. Different query optimization strategies needed

### Consolidation Options (If Required)
If consolidation becomes necessary:
```
Option 1: /search?response_type=facets_only
Option 2: /search?include_results=false
Option 3: Keep current design (RECOMMENDED)
```

---

## Typical User Flow

```
1. Page loads
   useSearchFilters() → /facets (static, all documents)
   Shows: All projects, versions, extensions

2. User selects filter "python"
   useFacetsWithFilters() → /facets?projects=python (debounced)
   Updates: Filter panel counts for selected filters

3. User searches "web framework"
   useMultiSelectSearch() → /search?q=web+framework&projects=python&include_facets=true
   Shows: Results + updated facet counts for "web framework" in "python"

4. User selects another filter "2.0"
   useFacetsWithFilters() → /facets?query=web+framework&projects=python&versions=2.0
   Updates: Facet counts for combined filters within query

5. User clears search
   useMultiSelectSearch() disabled
   useFacetsWithFilters() → /facets?projects=python&versions=2.0
   Shows: Filter counts for selected filters only
```

---

## Performance Notes

- **useSearchFilters()**: Called once, cached 5 minutes → cheap
- **useFacetsWithFilters()**: Debounced 500ms, called on filter change → moderate cost
- **useMultiSelectSearch()**: Called per search/pagination → more expensive
- All three use identical facet computation → consistent results

