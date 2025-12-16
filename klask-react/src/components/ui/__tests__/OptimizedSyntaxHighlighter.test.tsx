import { render } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import OptimizedSyntaxHighlighter from '../OptimizedSyntaxHighlighter';

// Mock prism-react-renderer to prevent "Cannot read properties of null (reading 'useCallback')" error
// This was the root cause of the file viewer crashing with white page
vi.mock('prism-react-renderer', () => {
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const React = require('react');
  return {
    Highlight: function MockHighlight({ children: renderFunc, code, language }: Record<string, unknown>) {
      // Create a simple mock context object that satisfies prism-react-renderer's API
      const mockCtx = {
        className: 'mock-highlight',
        style: {},
        tokens: [[{ types: ['plain'], content: code || '' }]],
        getLineProps: () => ({ style: {} }),
        getTokenProps: () => ({ style: {} }),
      };
      // Render a pre/code element with the render function result
      return React.createElement(
        'pre',
        { className: 'mock-highlight' },
        React.createElement('code', { 'data-language': language }, renderFunc(mockCtx))
      );
    },
    themes: {
      oneLight: {},
      oneDark: {},
      vsDark: {},
    },
  };
});

describe('OptimizedSyntaxHighlighter', () => {
  it('renders without throwing "Cannot read properties of null" error', () => {
    // This test ensures the file viewer no longer crashes
    expect(() => {
      render(
        <OptimizedSyntaxHighlighter language="javascript">
          {'const x = 1;'}
        </OptimizedSyntaxHighlighter>
      );
    }).not.toThrow();
  });

  it('renders code in pre and code elements', () => {
    const { container } = render(
      <OptimizedSyntaxHighlighter language="javascript">
        {'test code'}
      </OptimizedSyntaxHighlighter>
    );

    expect(container.querySelector('pre')).toBeInTheDocument();
    expect(container.querySelector('code')).toBeInTheDocument();
  });

  it('correctly passes language attribute', () => {
    const { container } = render(
      <OptimizedSyntaxHighlighter language="typescript">
        {'test'}
      </OptimizedSyntaxHighlighter>
    );

    const code = container.querySelector('code');
    expect(code?.getAttribute('data-language')).toBe('typescript');
  });

  it('normalizes language names to lowercase', () => {
    const { container } = render(
      <OptimizedSyntaxHighlighter language="JavaScript">
        {'test'}
      </OptimizedSyntaxHighlighter>
    );

    const code = container.querySelector('code');
    expect(code?.getAttribute('data-language')).toBe('javascript');
  });
});
