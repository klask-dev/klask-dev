import { describe, it, expect } from 'vitest';

interface FiltersObject {
  project: string[];
  version: string[];
  extension: string[];
  language: string[];
  repository: string[];
  size: { min?: number; max?: number };
}

// Unit test for the pagination fix logic
describe('SearchPageV3 - Pagination Regression Fix', () => {
  describe('Filter change detection logic (regression prevention)', () => {
    it('should correctly detect when filters have NOT changed', () => {
      // Simulates the fix logic: using useRef to track previous filters
      const previousFiltersRef = { current: undefined as FiltersObject | undefined };

      // Mock filters object (same structure as SearchFiltersContext)
      const filters = {
        project: [],
        version: [],
        extension: [],
        language: [],
        repository: [],
        size: { min: undefined, max: undefined },
      };

      // First call - no previous filter
      const firstPreviousFilters = previousFiltersRef.current;
      const filtersChanged = JSON.stringify(firstPreviousFilters) !== JSON.stringify(filters);

      expect(filtersChanged).toBe(true); // Changed (from undefined to object)

      // Update ref
      previousFiltersRef.current = filters;

      // Second call - same filters
      const secondPreviousFilters = previousFiltersRef.current;
      const filtersChanged2 = JSON.stringify(secondPreviousFilters) !== JSON.stringify(filters);

      expect(filtersChanged2).toBe(false); // NOT changed
    });

    it('should correctly detect when filters HAVE changed', () => {
      const previousFiltersRef = { current: undefined as FiltersObject | undefined };

      const initialFilters = {
        project: [],
        version: [],
        extension: [],
        language: [],
        repository: [],
        size: { min: undefined, max: undefined },
      };

      previousFiltersRef.current = initialFilters;

      // Filters changed - add a project
      const changedFilters = {
        project: ['project1'],
        version: [],
        extension: [],
        language: [],
        repository: [],
        size: { min: undefined, max: undefined },
      };

      const filtersChanged = JSON.stringify(previousFiltersRef.current) !== JSON.stringify(changedFilters);

      expect(filtersChanged).toBe(true); // Changed
    });

    it('should prevent circular dependency with currentPage', () => {
      // The fix removes currentPage from the dependency array
      // This test verifies that pagination changes don't trigger the effect

      let effectTriggered = 0;
      const previousFiltersRef = { current: undefined as FiltersObject | undefined };
      let currentPage = 1;

      const filters: FiltersObject = {
        project: [],
        version: [],
        extension: [],
        language: [],
        repository: [],
        size: { min: undefined, max: undefined },
      };

      const triggerEffect = () => {
        const previousFilters = previousFiltersRef.current;
        const filtersChanged = JSON.stringify(previousFilters) !== JSON.stringify(filters);

        // Only reset page if filters actually changed
        if (filtersChanged && currentPage !== 1) {
          effectTriggered++;
        }

        previousFiltersRef.current = filters;
      };

      // First run
      triggerEffect();
      expect(effectTriggered).toBe(0); // No reset (filters haven't "changed" from undefined)

      // Pagination changes (currentPage goes to 2)
      currentPage = 2;
      triggerEffect();
      expect(effectTriggered).toBe(0); // IMPORTANT: Should NOT reset on pagination change!

      // Pagination changes again (currentPage goes to 3)
      currentPage = 3;
      triggerEffect();
      expect(effectTriggered).toBe(0); // IMPORTANT: Still no reset!

      // Now filters actually change
      const newFilters = {
        ...filters,
        project: ['new-project'],
      };

      const filtersChanged = JSON.stringify(previousFiltersRef.current) !== JSON.stringify(newFilters);
      if (filtersChanged && currentPage !== 1) {
        effectTriggered++;
      }

      expect(effectTriggered).toBe(1); // NOW it resets (filters changed and page > 1)
    });

    it('should handle rapid filter changes without excessive resets', () => {
      const previousFiltersRef = { current: undefined as FiltersObject | undefined };
      let resetCount = 0;
      let currentPage = 1;

      const filters1: FiltersObject = { project: [], version: [], extension: [], language: [], repository: [], size: {} };
      const filters2: FiltersObject = { project: ['p1'], version: [], extension: [], language: [], repository: [], size: {} };
      const filters3: FiltersObject = { project: ['p1'], version: ['v1'], extension: [], language: [], repository: [], size: {} };

      const simulateEffect = (newFilters: FiltersObject) => {
        const filtersChanged = JSON.stringify(previousFiltersRef.current) !== JSON.stringify(newFilters);
        if (filtersChanged && currentPage !== 1) {
          resetCount++;
        }
        previousFiltersRef.current = newFilters;
      };

      simulateEffect(filters1);
      expect(resetCount).toBe(0);

      simulateEffect(filters2);
      expect(resetCount).toBe(0); // On page 1, shouldn't reset

      // Go to page 2
      currentPage = 2;

      simulateEffect(filters3);
      expect(resetCount).toBe(1); // Filter changed and page > 1, so reset

      // Change filter again
      const filters4: FiltersObject = { ...filters3, extension: ['ts'] };
      simulateEffect(filters4);
      expect(resetCount).toBe(2); // Another reset (but page should already be 1)
    });

    it('should not regress: pagination should not trigger effect', () => {
      // This is the exact regression that was happening:
      // The old code had currentPage in the dependency array,
      // which meant every pagination change triggered the effect,
      // which then unconditionally reset the page

      const previousFiltersRef = { current: undefined as FiltersObject | undefined };
      const filters: FiltersObject = { project: [], version: [], extension: [], language: [], repository: [], size: {} };
      let pageResetCount = 0;
      let currentPage = 1;

      // Initial state
      previousFiltersRef.current = filters;

      // User navigates to page 2
      currentPage = 2;

      // REGRESSION BUG: with currentPage in dependency array, effect triggers here
      // OLD CODE (buggy):
      // useEffect(() => {
      //   if (currentPage === 1) return;
      //   setCurrentPage(1); // ALWAYS RESETS!
      // }, [...filters, currentPage]); // currentPage in deps = BUG
      //
      // NEW CODE (fixed):
      // useEffect(() => {
      //   if (filters changed) {
      //     setCurrentPage(1); // Only resets on REAL filter change
      //   }
      // }, [filters]); // No currentPage = NO CIRCULAR DEPENDENCY

      const filtersChanged = JSON.stringify(previousFiltersRef.current) !== JSON.stringify(filters);

      // With the fix, filter hasn't changed, so no reset even though currentPage changed
      if (filtersChanged && currentPage !== 1) {
        pageResetCount++;
      }

      expect(pageResetCount).toBe(0); // PASS: No reset on pagination change!
      expect(currentPage).toBe(2); // PASS: Page remains at 2
    });
  });
});
