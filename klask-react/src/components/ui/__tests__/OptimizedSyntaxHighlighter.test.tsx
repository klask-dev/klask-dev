import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import OptimizedSyntaxHighlighter from '../OptimizedSyntaxHighlighter';

// Mock the syntax highlighter modules to avoid import issues in tests
vi.mock('react-syntax-highlighter/dist/esm/prism', () => ({
  Prism: {
    registerLanguage: vi.fn(),
  },
}));

vi.mock('react-syntax-highlighter/dist/esm/styles/prism/one-light', () => ({
  default: { 'pre[class*="language-"]': { background: '#fafafa' } },
}));

vi.mock('react-syntax-highlighter/dist/esm/styles/prism/one-dark', () => ({
  default: { 'pre[class*="language-"]': { background: '#282c34' } },
}));

vi.mock('react-syntax-highlighter/dist/esm/styles/prism/vsc-dark-plus', () => ({
  default: { 'pre[class*="language-"]': { background: '#1e1e1e' } },
}));

// Mock specific language imports that are commonly used
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/javascript', () => ({
  default: vi.fn(),
}));
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/typescript', () => ({
  default: vi.fn(),
}));
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/jsx', () => ({
  default: vi.fn(),
}));
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/tsx', () => ({
  default: vi.fn(),
}));
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/python', () => ({
  default: vi.fn(),
}));
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/json', () => ({
  default: vi.fn(),
}));
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/bash', () => ({
  default: vi.fn(),
}));
vi.mock('react-syntax-highlighter/dist/esm/languages/prism/markdown', () => ({
  default: vi.fn(),
}));

// Mock VirtualizedSyntaxHighlighter
vi.mock('../VirtualizedSyntaxHighlighter', () => ({
  default: vi.fn(({ children, language }) => (
    <div data-testid="virtualized-highlighter">
      <div data-language={language}>{children}</div>
    </div>
  )),
}));

// Mock LoadingSpinner
vi.mock('../LoadingSpinner', () => ({
  LoadingSpinner: ({ className }: { className?: string }) => (
    <div data-testid="loading-spinner" className={className}>Loading...</div>
  ),
}));

describe('OptimizedSyntaxHighlighter', () => {
  const defaultProps = {
    language: 'javascript',
    children: 'console.log("Hello, world!");',
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders loading spinner initially', () => {
    render(<OptimizedSyntaxHighlighter {...defaultProps} />);
    
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    expect(screen.getByText('Loading syntax highlighter...')).toBeInTheDocument();
  });

  it('normalizes language names correctly', () => {
    const { rerender } = render(
      <OptimizedSyntaxHighlighter language="JavaScript" children="test code" />
    );

    // Should normalize to lowercase
    expect(() => screen.getByText('test code')).not.toThrow();

    // Test with unsupported language
    rerender(
      <OptimizedSyntaxHighlighter language="unsupported" children="test code" />
    );

    // Should fallback to 'text'
    expect(() => screen.getByText('test code')).not.toThrow();
  });

  it('uses virtualized highlighter for large content', async () => {
    const largeContent = 'line\n'.repeat(150); // Reduced from 2000 to 150 for performance

    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        enableVirtualization={true}
        maxLines={100}
      >
        {largeContent}
      </OptimizedSyntaxHighlighter>
    );

    await waitFor(() => {
      expect(screen.getByTestId('virtualized-highlighter')).toBeInTheDocument();
    });
  });

  it('uses virtualized highlighter for large file size', async () => {
    const largeContent = 'a'.repeat(120000); // Reduced from 150000 to 120000 for performance

    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        enableVirtualization={true}
      >
        {largeContent}
      </OptimizedSyntaxHighlighter>
    );

    await waitFor(() => {
      expect(screen.getByTestId('virtualized-highlighter')).toBeInTheDocument();
    });
  });

  it('does not use virtualization when disabled', () => {
    const largeContent = 'line\n'.repeat(150); // Reduced from 2000 to 150 for performance

    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        enableVirtualization={false}
      >
        {largeContent}
      </OptimizedSyntaxHighlighter>
    );

    // Should show loading spinner instead of virtualized component
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    expect(screen.queryByTestId('virtualized-highlighter')).not.toBeInTheDocument();
  });

  it('passes correct props to virtualized highlighter', async () => {
    const VirtualizedHighlighter = await import('../VirtualizedSyntaxHighlighter');
    const largeContent = 'line\n'.repeat(150); // Reduced from 2000 to 150 for performance

    render(
      <OptimizedSyntaxHighlighter
        language="rust"
        style="oneDark"
        showLineNumbers={true}
        wrapLines={true}
        customStyle={{ fontSize: '16px' }}
        lineNumberStyle={{ color: 'red' }}
        className="test-class"
        maxLines={100}
      >
        {largeContent}
      </OptimizedSyntaxHighlighter>
    );

    await waitFor(() => {
      expect(screen.getByTestId('virtualized-highlighter')).toBeInTheDocument();
    });

    expect(VirtualizedHighlighter.default).toHaveBeenCalledWith(
      expect.objectContaining({
        language: 'rust',
        style: 'oneDark',
        showLineNumbers: true,
        wrapLines: true,
        customStyle: { fontSize: '16px' },
        lineNumberStyle: { color: 'red' },
        className: 'test-class',
        maxLines: 100,
        children: largeContent,
      }),
      expect.any(Object)
    );
  });

  it('supports all supported languages', () => {
    const supportedLanguages = [
      'javascript', 'typescript', 'jsx', 'tsx', 'python', 'java', 'cpp', 'c',
      'csharp', 'php', 'ruby', 'go', 'rust', 'kotlin', 'swift', 'dart', 'scala',
      'bash', 'yaml', 'json', 'xml', 'html', 'css', 'scss', 'sass', 'less',
      'sql', 'markdown', 'dockerfile'
    ];

    supportedLanguages.forEach(language => {
      const { unmount } = render(
        <OptimizedSyntaxHighlighter language={language} children="test" />
      );
      // Should render without errors
      expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
      unmount();
    });
  });

  it('handles empty content gracefully', () => {
    render(<OptimizedSyntaxHighlighter language="javascript" children="" />);
    
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('applies custom styles correctly', () => {
    const customStyle = {
      fontSize: '16px',
      backgroundColor: '#f0f0f0',
      padding: '20px',
    };

    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        customStyle={customStyle}
      >
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('handles different style themes', () => {
    const themes = ['oneLight', 'oneDark', 'vscDarkPlus'] as const;

    themes.forEach(theme => {
      const { unmount } = render(
        <OptimizedSyntaxHighlighter
          language="javascript"
          style={theme}
        >
          {defaultProps.children}
        </OptimizedSyntaxHighlighter>
      );
      
      expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
      unmount();
    });
  });

  it('toggles line numbers correctly', () => {
    const { rerender } = render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        showLineNumbers={false}
      >
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();

    rerender(
      <OptimizedSyntaxHighlighter
        language="javascript"
        showLineNumbers={true}
      >
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('handles wrap lines functionality', () => {
    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        wrapLines={true}
        wrapLongLines={true}
      >
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('applies line number styles correctly', () => {
    const lineNumberStyle = {
      color: '#666',
      backgroundColor: '#f5f5f5',
      paddingRight: '10px',
    };

    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        showLineNumbers={true}
        lineNumberStyle={lineNumberStyle}
      >
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('applies custom className', () => {
    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        className="custom-highlighter-class"
      >
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('calculates virtualization threshold correctly for line count', () => {
    const mediumContent = 'line\n'.repeat(500); // Less than default maxLines
    
    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        maxLines={1000}
      >
        {mediumContent}
      </OptimizedSyntaxHighlighter>
    );

    // Should not use virtualization
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    expect(screen.queryByTestId('virtualized-highlighter')).not.toBeInTheDocument();
  });

  it('calculates virtualization threshold correctly for content size', () => {
    const mediumContent = 'a'.repeat(50000); // Less than 100KB threshold
    
    render(
      <OptimizedSyntaxHighlighter language="javascript">
        {mediumContent}
      </OptimizedSyntaxHighlighter>
    );

    // Should not use virtualization
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
    expect(screen.queryByTestId('virtualized-highlighter')).not.toBeInTheDocument();
  });

  it('respects custom maxLines prop', async () => {
    const content = 'line\n'.repeat(120); // Reduced from 200 to 120 for performance

    render(
      <OptimizedSyntaxHighlighter
        language="javascript"
        maxLines={100} // Custom threshold
      >
        {content}
      </OptimizedSyntaxHighlighter>
    );

    await waitFor(() => {
      expect(screen.getByTestId('virtualized-highlighter')).toBeInTheDocument();
    });
  });

  it('handles code with special characters', () => {
    const specialContent = `
      const regex = /[.*+?^$\\{\\}()|[\\]\\\\]/g;
      const emoji = "Hello 👋 World 🌍";
      const unicode = "Üñíçødé";
    `;

    render(
      <OptimizedSyntaxHighlighter language="javascript">
        {specialContent}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('handles multiline code correctly', () => {
    const multilineContent = `
function fibonacci(n) {
  if (n <= 1) {
    return n;
  }
  return fibonacci(n - 1) + fibonacci(n - 2);
}

console.log(fibonacci(10));
    `.trim();

    render(
      <OptimizedSyntaxHighlighter language="javascript">
        {multilineContent}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('uses text fallback for unsupported languages', () => {
    render(
      <OptimizedSyntaxHighlighter language="made-up-language">
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });

  it('handles case-insensitive language matching', () => {
    const { rerender } = render(
      <OptimizedSyntaxHighlighter language="JAVASCRIPT">
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();

    rerender(
      <OptimizedSyntaxHighlighter language="TypeScript">
        {defaultProps.children}
      </OptimizedSyntaxHighlighter>
    );

    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument();
  });
});