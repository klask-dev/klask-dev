# CrawlErrorDisplay Component

## Overview

The `CrawlErrorDisplay` component provides a user-friendly interface to display crawl errors from repository indexing operations. It shows errors in a visually distinct manner with full dark mode support and helpful troubleshooting tips.

## Components

### 1. `CrawlErrorDisplay`

The main error display component with full details and troubleshooting tips.

**Props:**
- `errorMessage` (string, required): The error message to display
- `occurredAt` (string, optional): ISO timestamp of when the error occurred
- `repositoryName` (string, optional): Name of the repository where the error occurred
- `onDismiss` (() => void, optional): Callback when user dismisses the error
- `className` (string, optional): Additional CSS classes
- `compact` (boolean, optional, default: false): Show compact view with expandable details

**Features:**
- Displays error message with icon and timestamp
- Shows repository name context
- Expandable/collapsible for long error messages (compact mode)
- Includes troubleshooting tips
- Optional dismiss button
- Full dark mode support

### 2. `InlineCrawlError`

A compact inline error display suitable for cards and list items.

**Props:**
- `errorMessage` (string, required): The error message to display
- `onClick` (() => void, optional): Callback when error is clicked

**Features:**
- Compact, single-line display (truncated at 60 chars)
- "Click for details" hint when onClick is provided
- Dark mode support

## Usage Examples

### Basic Usage (Full Display)

```tsx
import { CrawlErrorDisplay } from './components/repositories/CrawlErrorDisplay';

function MyComponent() {
  return (
    <CrawlErrorDisplay
      errorMessage="Failed to clone repository: Authentication required"
      occurredAt="2025-10-24T20:00:00Z"
      repositoryName="my-project"
    />
  );
}
```

### Compact Mode with Expand/Collapse

```tsx
<CrawlErrorDisplay
  errorMessage="Very long error message that will be truncated in compact mode..."
  occurredAt="2025-10-24T20:00:00Z"
  repositoryName="my-project"
  compact={true}
/>
```

### With Dismiss Callback

```tsx
const [showError, setShowError] = useState(true);

{showError && (
  <CrawlErrorDisplay
    errorMessage="GitLab API rate limit exceeded"
    occurredAt="2025-10-24T20:00:00Z"
    repositoryName="gitlab-group"
    onDismiss={() => setShowError(false)}
  />
)}
```

### Inline Error in Card

```tsx
import { InlineCrawlError } from './components/repositories/CrawlErrorDisplay';

function RepositoryCard() {
  const [showDetails, setShowDetails] = useState(false);

  return (
    <div>
      <InlineCrawlError
        errorMessage="Connection timeout while fetching repository"
        onClick={() => setShowDetails(true)}
      />

      {showDetails && (
        <CrawlErrorDisplay
          errorMessage="Connection timeout while fetching repository: check network connectivity"
          occurredAt="2025-10-24T20:00:00Z"
        />
      )}
    </div>
  );
}
```

### Display Multiple Errors

```tsx
function ErrorList({ errors }: { errors: CrawlProgress[] }) {
  return (
    <div className="space-y-3">
      {errors.map(error => (
        <CrawlErrorDisplay
          key={error.repository_id}
          errorMessage={error.error_message!}
          occurredAt={error.updated_at}
          repositoryName={error.repository_name}
          compact={true}
        />
      ))}
    </div>
  );
}
```

## Integration Points

### RepositoryCard

Errors are automatically displayed in `RepositoryCard` when crawl progress contains an error:

```tsx
{crawlProgress?.error_message && (
  <div className="mt-4">
    <InlineCrawlError errorMessage={crawlProgress.error_message} />
  </div>
)}
```

### RepositoriesPage

Page-level error alerts shown at the top:

```tsx
{activeProgress.filter(p => p.error_message).length > 0 && (
  <div className="space-y-3">
    {activeProgress
      .filter(p => p.error_message)
      .map(progress => (
        <CrawlErrorDisplay
          key={progress.repository_id}
          errorMessage={progress.error_message!}
          occurredAt={progress.updated_at}
          repositoryName={progress.repository_name}
          compact={true}
        />
      ))
    }
  </div>
)}
```

## Visual Design

### Color Scheme

**Light Mode:**
- Background: `bg-red-50` with `border-red-200`
- Text: `text-red-700` to `text-red-800`
- Icons: `text-red-600`
- Hover: `hover:bg-red-100`

**Dark Mode:**
- Background: `dark:bg-red-900/20` with `dark:border-red-800`
- Text: `dark:text-red-200` to `dark:text-red-300`
- Icons: `dark:text-red-400`
- Hover: `dark:hover:bg-red-900/30`

### Accessibility

- Clear visual distinction with icon
- High contrast text
- Keyboard accessible (dismiss button, expand/collapse)
- Screen reader friendly with semantic HTML

## Error Types & Common Messages

Common error messages you might encounter:

1. **Authentication Errors:**
   - "Failed to authenticate with repository: invalid token"
   - "GitLab API: 401 Unauthorized"

2. **Network Errors:**
   - "Connection timeout while fetching repository"
   - "DNS resolution failed for repository URL"

3. **Permission Errors:**
   - "Access denied: insufficient permissions"
   - "Repository not found or access denied"

4. **Rate Limiting:**
   - "GitLab API rate limit exceeded, retry after X seconds"
   - "GitHub API: too many requests"

5. **Repository Issues:**
   - "Invalid repository URL format"
   - "Branch 'main' not found in repository"

## Troubleshooting Tips (Auto-displayed)

When expanded, the component shows these troubleshooting tips:
- Check if the repository URL is accessible
- Verify access token permissions if using private repositories
- Ensure network connectivity to the repository host
- Check repository logs for more details

## Backend Integration

The component expects error data from the backend's `CrawlProgressInfo` structure:

```rust
pub struct CrawlProgressInfo {
    pub repository_id: Uuid,
    pub repository_name: String,
    pub status: CrawlStatus,
    pub error_message: Option<String>,  // ← This field
    pub updated_at: DateTime<Utc>,
    // ...
}
```

When a crawl fails, the backend sets the error using:

```rust
progress.set_error("Error message here".to_string());
```

This automatically:
1. Sets `error_message` field
2. Updates `status` to `CrawlStatus::Failed`
3. Records `completed_at` timestamp
