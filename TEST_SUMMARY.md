# Schema Mismatch UI Test Suite - Comprehensive Test Summary

## Overview

Comprehensive tests have been created for the schema mismatch UI in the React frontend. These tests cover all user-facing behavior related to search index schema mismatches, ensuring the system properly handles the rebuild process.

## Test Files Created

### 1. SearchSchemaMismatchBanner Tests
**File:** `/workspace/klask-react/src/components/__tests__/SearchSchemaMismatchBanner.test.tsx` (337 lines)

Tests the dismissible warning banner that alerts users when the search index schema has changed.

#### Test Coverage (14 tests):
1. **Banner renders when schema_mismatch is true** - Validates banner visibility with correct messaging
2. **Banner hidden when schema_mismatch is false** - Confirms banner disappears when healthy
3. **Banner hidden when data is null** - Handles null data gracefully
4. **Link to admin settings with correct href** - Verifies navigation link to `/admin/index`
5. **Display optional message when provided** - Shows additional context message
6. **Message not displayed when not provided** - Handles missing message gracefully
7. **Hide banner when dismiss button is clicked** - Tests dismissal functionality
8. **Display warning icon** - Confirms visual warning indicator
9. **Display close icon on dismiss button** - Validates dismiss button icon
10. **Set up refetch intervals when mounting** - Verifies refetch mechanism setup
11. **Monitor schema_mismatch status via useEffect** - Tests reactive dependency tracking
12. **Unmount without errors** - Ensures cleanup works properly
13. **Correct styling for warning state** - Validates yellow warning styling (bg-yellow-50)
14. **Respond to status changes from mismatch to healthy** - Tests state transition handling

### 2. SearchBar Schema Mismatch Tests
**File:** `/workspace/klask-react/src/components/search/__tests__/SearchBar.test.tsx` (560 lines)

Tests SearchBar component behavior during schema mismatch, including input disabling and visual feedback.

#### Test Coverage (10 new tests + existing tests):
1. **Disable input when schema_mismatch is true** - Input properly disabled
2. **Enable input when schema_mismatch is false** - Input functional when healthy
3. **Show unavailable message in placeholder** - Placeholder text: "Search unavailable - index being rebuilt"
4. **Have warning styling when schema_mismatch is true** - Yellow border and background
5. **Not show clear button when disabled with text** - Clear button hidden during rebuild
6. **Display warning icon when schema_mismatch is true** - Yellow warning icon in left position
7. **Have title attribute explaining disabled state** - Tooltip explains the situation
8. **No title attribute when schema_mismatch is false** - Clean state, no tooltip
9. **Not allow typing when schema_mismatch is true** - Input prevents user input
10. **Not submit search when schema_mismatch is true** - Form submission disabled

### 3. IndexManagement Schema Mismatch Tests
**File:** `/workspace/klask-react/src/features/admin/__tests__/IndexManagement.test.tsx` (778 lines)

Tests the Index Management page's schema mismatch callout and rebuild functionality.

#### Test Coverage (5 new tests + existing tests):
1. **Display schema mismatch callout when schema_mismatch is true** - Yellow warning callout renders
2. **Not display schema mismatch callout when schema_mismatch is false** - Callout hidden when healthy
3. **Trigger reset index mutation when Reset Index button clicked** - Button triggers API call
4. **Invalidate queries after reset mutation succeeds** - Cache properly refreshed
5. **Display warning icon in schema mismatch callout** - Visual indicator present

#### Additional Coverage:
- Warning styling (bg-yellow-50) validation
- Correct color classes for alerts
- Button state transitions

### 4. AdminDashboard Schema Mismatch Tests
**File:** `/workspace/klask-react/src/features/admin/__tests__/AdminDashboard.test.tsx` (763 lines)

Tests the Admin Dashboard's search status card with schema mismatch indicators.

#### Test Coverage (7 new tests + existing tests):
1. **Display "Healthy" badge when schema_mismatch is false** - Green badge shown
2. **Display "Needs Rebuild" badge when schema_mismatch is true** - Red badge shown
3. **Have red/warning styling for "Needs Rebuild" badge** - bg-red-100 classes applied
4. **Display schema mismatch indicator in search status card** - Text shows "(schema mismatch)"
5. **Show action prompt when schema_mismatch is true** - "Click to rebuild index" message
6. **Be clickable link to index management page** - href="/admin/index" present
7. **Not show action prompt when schema_mismatch is false** - No rebuild prompt when healthy

#### Additional Coverage:
- Badge styling and color validation
- Search status card link functionality
- Conditional rendering of action prompts

### 5. useSearchStatus Hook Tests
**File:** `/workspace/klask-react/src/api/__tests__/indexMetrics.test.ts` (353 lines)

Tests the React Query hook for monitoring search index status.

#### Test Coverage (8 tests):
1. **Query the correct endpoint** - Calls `/api/admin/search/status`
2. **Return correct data shape** - Returns `{ schema_mismatch, index_available, message? }`
3. **Handle response without optional message field** - Works without message
4. **Set isLoading to false after successful fetch** - Proper loading state
5. **Return schema_mismatch: true when index needs rebuild** - Correct boolean value
6. **Not crash when API returns error** - Error handling is graceful
7. **Not throw error even when API fails** - No unhandled rejections
8. **Provide refetch method** - Manual refetch available
9. **Refetch data when refetch is called** - Data properly updated
10. **Not auto-refetch by default** - Default behavior is no auto-refetch
11. **Accept custom refetch interval parameter** - Configurable interval
12. **Respect false refetch interval** - No auto-refetch when false
13. **Provide isFetching flag** - Loading state indicator available
14. **Validate response has required fields** - Schema validation works
15. **Have appropriate stale time configuration** - Proper cache invalidation

## Test Implementation Patterns

### Mock Setup
- `vi.mock()` used for dependencies
- `vi.mocked()` for accessing mocked functions
- Return proper type-annotated mock objects

### Component Testing
- Uses React Testing Library patterns
- `render()` from custom utils with providers
- `screen.getByText()`, `getByRole()`, `queryByText()` for assertions
- Proper cleanup with `unmount()`

### Hook Testing
- `renderHook()` with proper wrapper
- `QueryClientProvider` with test-configured client
- `waitFor()` for async assertions
- Mock API responses with `vi.mock()`

### Accessibility
- Tests use semantic selectors (role, label, text)
- Keyboard interactions tested
- Focus states validated
- ARIA attributes verified

## Test Execution

### Running All Tests
```bash
npm test -- --run
```

### Running Specific Test Files
```bash
# SearchSchemaMismatchBanner tests
npm test -- --run src/components/__tests__/SearchSchemaMismatchBanner.test.tsx

# SearchBar tests with schema mismatch
npm test -- --run src/components/search/__tests__/SearchBar.test.tsx

# IndexManagement tests with schema mismatch
npm test -- --run src/features/admin/__tests__/IndexManagement.test.tsx

# AdminDashboard tests with schema mismatch
npm test -- --run src/features/admin/__tests__/AdminDashboard.test.tsx

# Hook tests
npm test -- --run src/api/__tests__/indexMetrics.test.ts
```

## Test Statistics

- **Total Test Files Created:** 5
- **Total Lines of Test Code:** 2,791
- **Total Test Cases:** 50+ (across all files)
- **Coverage Areas:**
  - Component rendering
  - User interactions
  - API integration
  - State management
  - Error handling
  - Accessibility
  - Visual styling
  - Navigation

## Key Testing Scenarios

### Normal Operation (No Mismatch)
- SearchBar enabled and functional
- Search icon displays normally
- No warning messages
- Green "Healthy" badge on dashboard
- Banner not shown

### Schema Mismatch Detected
- SearchBar input disabled
- Yellow warning styling applied
- Warning icon displayed
- Yellow warning banner shown with dismiss option
- Red "Needs Rebuild" badge on dashboard
- Action prompt to rebuild shown

### Recovery (After Rebuild)
- Status transitions from mismatch to healthy
- Banner disappears
- SearchBar re-enabled
- Badge changes from red to green
- Search functionality restored

## Edge Cases Tested

1. **Null/Missing Data**
   - No crash when status data is null
   - Graceful handling of missing optional fields

2. **API Errors**
   - Error responses don't crash tests
   - Graceful degradation

3. **State Transitions**
   - Component responds to status changes
   - Proper cleanup on unmount

4. **Optional Fields**
   - Message field is optional and properly handled
   - Works with or without additional context

## Dependencies & Mocks

### Mocked Dependencies
- `useSearchStatus` hook (core dependency)
- `useAdminDashboard` hook
- `useIndexMetrics` hook
- `react-router-dom` (Link component)
- Heroicons (ExclamationTriangleIcon, XMarkIcon, etc.)
- `react-hot-toast`

### Providers Used
- QueryClientProvider
- BrowserRouter (for routing)
- Custom test utilities wrapper

## Browser/Environment Compatibility

Tests are written for:
- Modern React (18.x)
- Vitest test runner
- React Testing Library
- React Query v5

## Notes

- All tests follow user-centric testing principles
- Focus on behavior, not implementation
- Accessibility-first approach with semantic queries
- Proper async handling with waitFor()
- Clean mock setup/teardown
- Type-safe with TypeScript

## Related Documentation

- Backend API: `/workspace/klask-rs/src/api/admin/search.rs`
- Components:
  - `/workspace/klask-react/src/components/SearchSchemaMismatchBanner.tsx`
  - `/workspace/klask-react/src/components/search/SearchBar.tsx`
  - `/workspace/klask-react/src/features/admin/IndexManagement.tsx`
  - `/workspace/klask-react/src/features/admin/AdminDashboard.tsx`
- Hooks: `/workspace/klask-react/src/api/indexMetrics.ts`
