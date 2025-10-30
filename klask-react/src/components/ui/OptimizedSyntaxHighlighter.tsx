import React, { useMemo } from 'react';
import { Highlight, themes } from 'prism-react-renderer';

/**
 * OptimizedSyntaxHighlighter
 *
 * Uses prism-react-renderer for fast, reliable syntax highlighting.
 * Supports 100+ languages out of the box with proper theme support.
 */

interface OptimizedSyntaxHighlighterProps {
  children: string;
  language: string;
  style?: 'oneLight' | 'oneDark' | 'vscDarkPlus';
  showLineNumbers?: boolean;
  wrapLines?: boolean;
  wrapLongLines?: boolean;
  customStyle?: React.CSSProperties;
  lineNumberStyle?: React.CSSProperties;
  className?: string;
  enableVirtualization?: boolean;
  maxLines?: number;
  lineHeight?: number;
  containerHeight?: number;
}

// Theme mapping
const getTheme = (styleName: string, isDarkMode: boolean) => {
  const themeMap: Record<string, any> = {
    oneLight: themes.oneLight,
    oneDark: themes.oneDark,
    vscDarkPlus: themes.vsDark,
  };

  // Default to system theme if not specified
  if (!styleName) {
    return isDarkMode ? themes.vsDark : themes.oneLight;
  }

  return themeMap[styleName] || (isDarkMode ? themes.vsDark : themes.oneLight);
};

const OptimizedSyntaxHighlighter: React.FC<OptimizedSyntaxHighlighterProps> = ({
  children,
  language,
  style,
  showLineNumbers = true,
  wrapLongLines = false,
  customStyle = {},
  lineNumberStyle = {},
  className = '',
}) => {
  // Detect dark mode
  const isDarkMode = useMemo(() => {
    if (typeof window === 'undefined') return true;
    return (
      window.matchMedia &&
      window.matchMedia('(prefers-color-scheme: dark)').matches
    );
  }, []);

  const theme = useMemo(() => getTheme(style || '', isDarkMode), [style, isDarkMode]);

  // Check if file is too large
  if (children.length > 100000) {
    return (
      <div
        style={customStyle}
        className={`p-4 bg-gray-50 dark:bg-slate-950 border rounded ${className}`}
      >
        <div className="mb-4 p-2 bg-yellow-100 dark:bg-yellow-900 border-l-4 border-yellow-400 text-yellow-700 dark:text-yellow-200">
          <p className="font-medium">Large File</p>
          <p className="text-sm">
            This file is very large ({(children.length / 1024).toFixed(1)}KB).
            Syntax highlighting has been disabled for performance.
          </p>
        </div>
        <pre className="whitespace-pre-wrap text-sm font-mono overflow-auto text-slate-700 dark:text-slate-300 bg-white dark:bg-slate-900 p-4 rounded">
          {children}
        </pre>
      </div>
    );
  }

  return (
    <Highlight theme={theme} code={children} language={language.toLowerCase()}>
      {({ className: highlightClassName, style: highlightStyle, tokens, getLineProps, getTokenProps }) => (
        <pre
          className={`${highlightClassName} ${className} p-4 rounded overflow-x-auto`}
          style={{
            ...highlightStyle,
            ...customStyle,
            margin: 0,
            fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace',
            fontSize: '0.875rem',
            lineHeight: '1.5',
          }}
        >
          <code>
            {tokens.map((line, lineIndex) => (
              <div
                key={lineIndex}
                {...getLineProps({ line })}
                style={{
                  display: 'flex',
                  whiteSpace: wrapLongLines ? 'pre-wrap' : 'pre',
                  wordBreak: wrapLongLines ? 'break-word' : 'normal',
                }}
              >
                {showLineNumbers && (
                  <span
                    style={{
                      display: 'inline-block',
                      width: '3rem',
                      marginRight: '1rem',
                      textAlign: 'right',
                      userSelect: 'none',
                      opacity: 0.5,
                      ...lineNumberStyle,
                    }}
                  >
                    {lineIndex + 1}
                  </span>
                )}
                <span style={{ flex: 1 }}>
                  {line.map((token, tokenIndex) => (
                    <span key={tokenIndex} {...getTokenProps({ token })} />
                  ))}
                </span>
              </div>
            ))}
          </code>
        </pre>
      )}
    </Highlight>
  );
};

export default OptimizedSyntaxHighlighter;
