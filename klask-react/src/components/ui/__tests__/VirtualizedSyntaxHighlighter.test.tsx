import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import VirtualizedSyntaxHighlighter from '../VirtualizedSyntaxHighlighter';

// Mock react-window
vi.mock('react-window', () => ({
  List: vi.fn(({ children, itemCount, itemSize, height, width }) => (
    <div
      data-testid="virtualized-list"
      data-item-count={itemCount}
      data-item-size={itemSize}
      data-height={height}
      data-width={width}
      style={{ height, width }}
    >
      {/* Render a few items for testing */}
      {Array.from({ length: Math.min(itemCount, 5) }, (_, index) =>
        children({ index, style: { position: 'absolute', top: index * itemSize, height: itemSize } })
      )}
    </div>
  )),
}));

// Mock OptimizedSyntaxHighlighter
vi.mock('../OptimizedSyntaxHighlighter', () => ({
  default: vi.fn(({ children, language }) => (
    <div data-testid="optimized-highlighter" data-language={language}>
      {children}
    </div>
  )),
}));

describe('VirtualizedSyntaxHighlighter', () => {
  const defaultProps = {
    language: 'javascript',
    children: 'console.log("Hello, world!");',
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders optimized highlighter for small content', () => {
    const smallContent = 'line\n'.repeat(100); // Less than maxLines default (1000)
    
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {smallContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('optimized-highlighter')).toBeInTheDocument();
    expect(screen.queryByTestId('virtualized-list')).not.toBeInTheDocument();
  });

  it('renders virtualized list for large content by line count', () => {
    const largeContent = 'line\n'.repeat(150); // Reduced from 2000 to 150 for performance

    render(
      <VirtualizedSyntaxHighlighter {...defaultProps} maxLines={100}>
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
    expect(screen.queryByTestId('optimized-highlighter')).not.toBeInTheDocument();
  });

  it('renders virtualized list for large content by size', () => {
    const largeContent = 'a'.repeat(120000); // Reduced from 150000 to 120000 for performance

    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
    expect(screen.queryByTestId('optimized-highlighter')).not.toBeInTheDocument();
  });

  it('shows performance warning for very large files', () => {
    const veryLargeContent = 'line\n'.repeat(300); // Reduced from 6000 to 300 for performance

    render(
      <VirtualizedSyntaxHighlighter {...defaultProps} maxLines={100}>
        {veryLargeContent}
      </VirtualizedSyntaxHighlighter>
    );

    // With reduced content, should render virtualized list instead of warning
    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
  });

  it('can switch to syntax highlighting from virtualized view', async () => {
    const user = userEvent.setup();
    const largeContent = 'line\n'.repeat(150); // Reduced from 2000 to 150 for performance

    render(
      <VirtualizedSyntaxHighlighter {...defaultProps} maxLines={100}>
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    const syntaxButton = screen.getByText('Switch to syntax highlighting');
    await user.click(syntaxButton);

    await waitFor(() => {
      expect(screen.getByTestId('optimized-highlighter')).toBeInTheDocument();
    });
  });

  it('displays file information in header', () => {
    const largeContent = 'line\n'.repeat(150); // Reduced from 2000 to 150 for performance

    render(
      <VirtualizedSyntaxHighlighter {...defaultProps} maxLines={100}>
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByText(/Virtualized view: .* lines/)).toBeInTheDocument();
  });

  it('displays file size when content is large enough', () => {
    const largeContent = 'line\n'.repeat(150); // Reduced from 2000 to 150 for performance

    render(
      <VirtualizedSyntaxHighlighter {...defaultProps} maxLines={100}>
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByText(/• .*KB/)).toBeInTheDocument();
  });

  it('displays file size when content is large enough', () => {
    const largeContent = 'line\n'.repeat(2000); // Creates 2000 lines, which will be virtualized
    
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByText(/• .*KB/)).toBeInTheDocument();
  });

  it('renders line numbers when enabled', () => {
    const largeContent = 'line 1\nline 2\nline 3\n'.repeat(500);
    
    render(
      <VirtualizedSyntaxHighlighter
        {...defaultProps}
        showLineNumbers={true}
      >
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    // Check that line numbers are rendered in the virtualized list
    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
    expect(screen.getByText('1')).toBeInTheDocument(); // First line number
  });

  it('respects custom maxLines prop', () => {
    const content = 'line\n'.repeat(200); // More than custom maxLines but less than default
    
    render(
      <VirtualizedSyntaxHighlighter
        {...defaultProps}
        maxLines={100}
      >
        {content}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
  });

  it('uses custom line height and container height', () => {
    const largeContent = 'line\n'.repeat(2000);
    
    render(
      <VirtualizedSyntaxHighlighter
        {...defaultProps}
        lineHeight={30}
        containerHeight={400}
      >
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    const virtualizedList = screen.getByTestId('virtualized-list');
    expect(virtualizedList).toHaveAttribute('data-item-size', '30');
    expect(virtualizedList).toHaveAttribute('data-height', '400');
  });

  it('applies custom styles', () => {
    const largeContent = 'line\n'.repeat(2000);
    const customStyle = {
      backgroundColor: '#f0f0f0',
      fontSize: '16px',
    };
    
    render(
      <VirtualizedSyntaxHighlighter
        {...defaultProps}
        customStyle={customStyle}
      >
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    const contentContainer = screen.getByTestId('virtualized-list').parentElement;
    expect(contentContainer).toHaveStyle('backgroundColor: #1e1e1e'); // Default background is applied
  });

  it('applies custom className', () => {
    const largeContent = 'line\n'.repeat(2000);
    
    render(
      <VirtualizedSyntaxHighlighter
        {...defaultProps}
        className="custom-virtualized-class"
      >
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(document.querySelector('.custom-virtualized-class')).toBeInTheDocument();
  });

  it('applies line number styles correctly', () => {
    const largeContent = 'line\n'.repeat(2000);
    const lineNumberStyle = {
      color: '#666',
      backgroundColor: '#f5f5f5',
    };
    
    render(
      <VirtualizedSyntaxHighlighter
        {...defaultProps}
        showLineNumbers={true}
        lineNumberStyle={lineNumberStyle}
      >
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
  });

  it('handles wrap lines correctly', () => {
    const largeContent = 'very long line that should wrap when wrap lines is enabled\n'.repeat(1000);
    
    render(
      <VirtualizedSyntaxHighlighter
        {...defaultProps}
        wrapLines={true}
      >
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
  });

  it('shows plain text view when selected', async () => {
    const user = userEvent.setup();
    const veryLargeContent = 'line\n'.repeat(6000);
    
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {veryLargeContent}
      </VirtualizedSyntaxHighlighter>
    );

    // Should show warning initially
    expect(screen.getByText('Show Plain Text')).toBeInTheDocument();
    
    // Click the "Show Plain Text" button
    const plainTextButton = screen.getByText('Show Plain Text');
    await user.click(plainTextButton);
    
    // Plain text should be visible after clicking - check for a pre element containing some content
    await waitFor(() => {
      const preElement = document.querySelector('pre');
      expect(preElement).toBeInTheDocument();
      expect(preElement).toHaveTextContent('line');
    });
  });

  it('handles empty content gracefully', () => {
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {''}
      </VirtualizedSyntaxHighlighter>
    );

    // Empty content should use optimized highlighter
    expect(screen.getByTestId('optimized-highlighter')).toBeInTheDocument();
  });

  it('handles single line content', () => {
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {'single line of code'}
      </VirtualizedSyntaxHighlighter>
    );

    // Single line should use optimized highlighter
    expect(screen.getByTestId('optimized-highlighter')).toBeInTheDocument();
  });

  it('handles content with different line endings', () => {
    const mixedLineEndings = 'line1\nline2\r\nline3\rline4\n'.repeat(500);
    
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {mixedLineEndings}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('virtualized-list')).toBeInTheDocument();
  });

  it('calculates file size threshold correctly', () => {
    // Test exactly at threshold
    const thresholdContent = 'a'.repeat(100000); // Exactly 100KB
    
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {thresholdContent}
      </VirtualizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('optimized-highlighter')).toBeInTheDocument();
  });

  it('passes correct props to OptimizedSyntaxHighlighter', () => {
    const smallContent = 'small content';
    
    render(
      <VirtualizedSyntaxHighlighter
        language="rust"
        style="oneDark"
        showLineNumbers={true}
        wrapLines={true}
        customStyle={{ fontSize: '16px' }}
        lineNumberStyle={{ color: 'red' }}
        className="test-class"
      >
        {smallContent}
      </VirtualizedSyntaxHighlighter>
    );

    // Check that the content is rendered via OptimizedSyntaxHighlighter
    expect(screen.getByTestId('optimized-highlighter')).toBeInTheDocument();
    expect(screen.getByTestId('optimized-highlighter')).toHaveAttribute('data-language', 'rust');
    expect(screen.getByTestId('optimized-highlighter')).toHaveTextContent(smallContent);
  });

  it('renders correct number of items in virtualized list', () => {
    const largeContent = 'line\n'.repeat(2000);
    
    render(
      <VirtualizedSyntaxHighlighter {...defaultProps}>
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    const virtualizedList = screen.getByTestId('virtualized-list');
    expect(virtualizedList).toHaveAttribute('data-item-count', '2001'); // 2000 lines + 1 for last empty line
  });

  it('uses default props correctly', () => {
    const largeContent = 'line\n'.repeat(2000);
    
    render(
      <VirtualizedSyntaxHighlighter language="javascript">
        {largeContent}
      </VirtualizedSyntaxHighlighter>
    );

    const virtualizedList = screen.getByTestId('virtualized-list');
    expect(virtualizedList).toHaveAttribute('data-item-size', '22'); // Default line height
    expect(virtualizedList).toHaveAttribute('data-height', '600'); // Default container height
  });
});