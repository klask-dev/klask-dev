# Klask API Architecture Analysis - Documentation Index

This directory contains comprehensive analysis of the `/search` and `/facets` API endpoints relationship in Klask.

## Quick Start

If you have **5 minutes**: Read `SEARCH_FACETS_QUICK_REFERENCE.md`

If you have **15 minutes**: Read `SEARCH_FACETS_QUICK_REFERENCE.md` + the Key Decision Point section

If you have **30 minutes**: Read `SEARCH_FACETS_ARCHITECTURE.md` 

If you have **1 hour+**: Read all three documents in order

---

## Documents Overview

### 1. SEARCH_FACETS_QUICK_REFERENCE.md
**Best for**: Quick lookup and decision making during development

**Contains**:
- One-page endpoint comparison table
- Frontend hooks comparison
- Decision flowchart: when to use /search vs /facets
- Typical user flow walkthrough
- Key performance notes
- Code references by location

**Sections**:
- Endpoint Purposes (comparison table)
- Frontend Hooks (usage matrix)
- Facet Data (structure vs values)
- Key Decision Point (choosing endpoints)
- Facet Collection Logic (self-dimension exclusion)
- Code References (4 main locations)
- Are They Redundant? (answer: NO)
- Should They Be Combined? (answer: NO)
- Consolidation Options (if required)
- Typical User Flow (5-step walkthrough)
- Performance Notes

---

### 2. SEARCH_FACETS_ARCHITECTURE.md
**Best for**: Understanding system behavior and data flows

**Contains**:
- ASCII system architecture diagram
- Full frontend-to-backend architecture visualization
- 5 detailed scenario walkthroughs with timelines
- Query decision tree (visual flowchart)
- Facet computation flow with aggregation logic
- Cache strategy breakdown
- Why consolidation would fail (examples)

**Sections**:
- System Architecture Overview (full diagram with data flow)
- Scenario 1: Page Load (initial filter display)
- Scenario 2: User Changes Filters (no active search)
- Scenario 3: User Searches with Filters
- Scenario 4: User Adds Another Filter During Search
- Scenario 5: User Clears Search
- Query Decision Tree (visual flowchart)
- Facet Computation Flow (per-dimension breakdown)
- Cache Strategy (independent caching per hook)
- Why NOT Single Endpoint? (consolidation problems)

---

### 3. SEARCH_FACETS_ANALYSIS.md
**Best for**: Deep understanding and architecture decisions

**Contains**:
- Comprehensive technical analysis (352 lines)
- Detailed code references (file:line format)
- Request/response structure breakdowns
- Facet computation logic explanation
- Frontend hook detailed comparison
- Data flow architecture with state management
- Detailed usage scenarios with API call counts
- Facet identity analysis (structure vs values)
- 7 architectural justification reasons
- Redundancy analysis
- Consolidation feasibility assessment
- 3 consolidation options with pros/cons

**Sections**:
1. Executive Summary
2. Backend API Endpoints Analysis (structures, characteristics, differences)
3. Facet Computation (identical logic, key insight on dimension exclusion)
4. Data Flow Architecture (three facet sources, merging strategy)
5. Detailed Usage Scenarios (3 scenarios: search only, filter only, search+filter)
6. Are Facets Identical? (structure YES, values NO explanation)
7. Why Both Endpoints? (7 architectural reasons)
8. Is There Redundancy? (NO - detailed analysis)
9. Consolidation Feasibility (technical vs recommended, 3 options)
10. Summary Code References (table format)
11. Conclusion

---

## Key Findings at a Glance

| Question | Answer |
|----------|--------|
| Are /search and /facets endpoints identical? | NO - different structures and data |
| Are they redundant? | NO - serve complementary purposes |
| Why both endpoints called? | Different lifecycle patterns: search → results; filter change → counts only |
| Difference in filter handling? | YES: /search has no validation, /facets has strict validation |
| Can they be consolidated? | Technically yes, but NOT recommended |
| Impact of consolidation? | Performance regression, UX degradation |

---

## Code Locations (File:Line Format)

**Backend Endpoints**:
- `/klask-rs/src/api/search.rs:130-217` — /search endpoint
- `/klask-rs/src/api/search.rs:219-317` — /facets endpoint
- `/klask-rs/src/api/search.rs:53-108` — Request/Response structures
- `/klask-rs/src/services/search.rs:955-1087` — Facet computation logic

**Frontend Hooks**:
- `/klask-react/src/hooks/useSearch.ts:109-127` — useSearchFilters()
- `/klask-react/src/hooks/useSearch.ts:130-211` — useMultiSelectSearch()
- `/klask-react/src/hooks/useSearch.ts:461-572` — useFacetsWithFilters()

**Frontend Context**:
- `/klask-react/src/contexts/SearchFiltersContext.tsx:1-150+` — Filter state management
- `/klask-react/src/features/search/SearchPageV3.tsx:102-127` — Usage example

**API Client**:
- `/klask-react/src/lib/api.ts:273-289` — API client search method

---

## Frontend Hook Usage Summary

### useMultiSelectSearch()
- **Called from**: SearchPageV3.tsx
- **Hits**: `/search` endpoint
- **Returns**: Results + query-specific facets
- **Triggered**: When user searches
- **Purpose**: Display search results with context-aware filter counts

### useFacetsWithFilters()
- **Called from**: SearchFiltersContext.tsx
- **Hits**: `/facets` endpoint
- **Returns**: Facets only
- **Triggered**: When filters change (debounced 500ms)
- **Purpose**: Update filter panel counts dynamically

### useSearchFilters()
- **Called from**: SearchFiltersContext.tsx
- **Hits**: `/facets` endpoint
- **Returns**: Static facets for all documents
- **Triggered**: Once on mount, cached 5 minutes
- **Purpose**: Show available filter options globally

---

## Architectural Decisions

### Why Separate Endpoints Work

1. **Performance**: /facets with limit=0 much faster than /search with results
2. **Caching**: Each has different cache requirements (5 min vs 60 sec vs 30 sec)
3. **UX**: Filter panel should update without fetching results
4. **Evolution**: Endpoints can be modified independently
5. **Clarity**: Clear separation: "retrieve docs" vs "aggregate counts"

### Why Consolidation Fails

1. Forces fetching results just to get filter counts
2. Filter panel updates would be expensive
3. Can't have independent cache expiry
4. Breaks existing API contract
5. Reduces architectural clarity

---

## Recommended Reading Order

**For Understanding Current State**:
1. SEARCH_FACETS_QUICK_REFERENCE.md (understand what each endpoint does)
2. SEARCH_FACETS_ARCHITECTURE.md (understand how they work together)
3. SEARCH_FACETS_ANALYSIS.md (understand why it's designed this way)

**For Making API Changes**:
1. SEARCH_FACETS_QUICK_REFERENCE.md (decision points)
2. SEARCH_FACETS_ARCHITECTURE.md (data flow impact)
3. SEARCH_FACETS_ANALYSIS.md (architectural implications)

**For New Developers**:
1. SEARCH_FACETS_QUICK_REFERENCE.md (10 minutes)
2. SEARCH_FACETS_ARCHITECTURE.md (scenarios section) (15 minutes)
3. Reference SEARCH_FACETS_ANALYSIS.md as needed

---

## FAQ

**Q: Should I always call /search when user searches?**
A: Yes, with `include_facets=true` to get context-specific counts.

**Q: Should I always call /facets when filters change?**
A: Yes, to update filter panel counts for the new filter combination.

**Q: Can I just use /search for everything?**
A: Technically yes, but performance would suffer (always fetching results).

**Q: Can I use /facets alone for all facet needs?**
A: Yes, but you lose search results display.

**Q: Why does /facets validate filters but /search doesn't?**
A: /facets is the core facet endpoint; /search is optional feature.

**Q: Are the facet counts from /search and /facets identical?**
A: No - they show counts for different contexts (query results vs filtered set).

**Q: Should I consolidate these endpoints?**
A: No - consolidation would cause performance regression.

---

## Additional Resources

- `CLAUDE.md` - Overall Klask development guide
- `.claude/` - AI agent prompts and workflows
- Backend tests: `klask-rs/tests/`
- Frontend tests: `klask-react/src/__tests__/`

---

## Document Statistics

| Document | Lines | Size | Focus |
|----------|-------|------|-------|
| SEARCH_FACETS_ANALYSIS.md | 352 | 12K | Comprehensive analysis |
| SEARCH_FACETS_ARCHITECTURE.md | 342 | 18K | Visual diagrams, scenarios |
| SEARCH_FACETS_QUICK_REFERENCE.md | 113 | 4.4K | Quick lookup, decisions |
| **Total** | **807** | **34K** | Complete documentation |

---

## Questions or Updates Needed?

These documents were generated as part of codebase analysis:
- Last updated: November 2, 2025
- Covers: klask-rs and klask-react current code
- Scope: /search and /facets API endpoints and their relationship

If code changes, please update these documents to reflect new behavior.

---

**Happy coding!**

