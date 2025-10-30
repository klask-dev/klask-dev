/**
 * SimpleSyntaxHighlighter
 *
 * A lightweight code highlighter that doesn't depend on react-syntax-highlighter
 * or refractor. It provides basic syntax highlighting using a tokenization approach.
 */

import React, { useMemo } from 'react';

interface SimpleSyntaxHighlighterProps {
  children: string;
  language?: string;
  showLineNumbers?: boolean;
  customStyle?: React.CSSProperties;
}

// Token types for syntax highlighting
type TokenType = 'keyword' | 'string' | 'comment' | 'number' | 'text';

interface Token {
  type: TokenType;
  value: string;
}

// Language-specific keywords
const keywords: Record<string, Set<string>> = {
  javascript: new Set([
    'function', 'const', 'let', 'var', 'if', 'else', 'for', 'while', 'return',
    'class', 'import', 'export', 'default', 'async', 'await', 'new', 'typeof',
    'instanceof', 'true', 'false', 'null', 'undefined', 'this', 'switch', 'case',
    'break', 'continue', 'do', 'try', 'catch', 'finally', 'throw', 'extends',
    'super', 'static', 'get', 'set', 'of', 'in', 'from', 'as',
  ]),
  typescript: new Set([
    'function', 'const', 'let', 'var', 'if', 'else', 'for', 'while', 'return',
    'class', 'interface', 'type', 'import', 'export', 'default', 'async', 'await',
    'new', 'typeof', 'instanceof', 'enum', 'namespace', 'declare', 'as', 'true',
    'false', 'null', 'undefined', 'this', 'switch', 'case', 'break', 'continue',
    'do', 'try', 'catch', 'finally', 'throw', 'extends', 'super', 'static',
    'get', 'set', 'of', 'in', 'from', 'abstract', 'public', 'private', 'protected',
    'readonly',
  ]),
  python: new Set([
    'def', 'class', 'if', 'elif', 'else', 'for', 'while', 'return', 'import',
    'from', 'as', 'with', 'try', 'except', 'finally', 'raise', 'pass', 'break',
    'continue', 'lambda', 'yield', 'and', 'or', 'not', 'in', 'is', 'True', 'False',
    'None', 'self', 'super', 'async', 'await',
  ]),
  default: new Set([]),
};

// Tokenize code by analyzing character by character
const tokenize = (code: string, language: string): Token[] => {
  const tokens: Token[] = [];
  const keywordSet = keywords[language.toLowerCase()] || keywords.default;
  let i = 0;

  while (i < code.length) {
    // Check for comments
    if (code[i] === '/' && code[i + 1] === '/') {
      let comment = '';
      while (i < code.length && code[i] !== '\n') {
        comment += code[i];
        i++;
      }
      tokens.push({ type: 'comment', value: comment });
      continue;
    }

    // Check for strings
    if (code[i] === '"' || code[i] === "'" || code[i] === '`') {
      const quote = code[i];
      let string = quote;
      i++;
      while (i < code.length && code[i] !== quote) {
        if (code[i] === '\\') {
          string += code[i] + code[i + 1];
          i += 2;
        } else {
          string += code[i];
          i++;
        }
      }
      if (i < code.length) {
        string += code[i];
        i++;
      }
      tokens.push({ type: 'string', value: string });
      continue;
    }

    // Check for numbers
    if (/\d/.test(code[i])) {
      let number = '';
      while (i < code.length && /[\d._]/.test(code[i])) {
        number += code[i];
        i++;
      }
      tokens.push({ type: 'number', value: number });
      continue;
    }

    // Check for keywords and identifiers
    if (/[a-zA-Z_$]/.test(code[i])) {
      let identifier = '';
      while (i < code.length && /[a-zA-Z0-9_$]/.test(code[i])) {
        identifier += code[i];
        i++;
      }
      if (keywordSet.has(identifier)) {
        tokens.push({ type: 'keyword', value: identifier });
      } else {
        tokens.push({ type: 'text', value: identifier });
      }
      continue;
    }

    // Everything else is text
    let text = '';
    while (i < code.length && !/[a-zA-Z0-9_$"'`\/\n]/.test(code[i])) {
      text += code[i];
      i++;
    }
    if (text) {
      tokens.push({ type: 'text', value: text });
    } else {
      tokens.push({ type: 'text', value: code[i] });
      i++;
    }
  }

  return tokens;
};

const SimpleSyntaxHighlighter: React.FC<SimpleSyntaxHighlighterProps> = ({
  children,
  language = 'text',
  showLineNumbers = false,
  customStyle = {},
}) => {
  // Detect dark mode
  const isDarkMode = useMemo(() => {
    if (typeof window === 'undefined') return true;
    return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
  }, []);

  // Color scheme based on theme
  const colors = isDarkMode
    ? {
        bg: '#1e293b',
        text: '#e2e8f0',
        keywords: '#74c0fc',
        strings: '#51cf66',
        comments: '#ff6b6b',
        numbers: '#ffa94d',
        lineNumbers: '#64748b',
      }
    : {
        bg: '#f1f5f9',
        text: '#1e293b',
        keywords: '#0c4a6e',
        strings: '#15803d',
        comments: '#991b1b',
        numbers: '#9a3412',
        lineNumbers: '#94a3b8',
      };

  const lines = useMemo(() => children.split('\n'), [children]);

  // Render a token with appropriate styling
  const renderToken = (token: Token, key: number) => {
    const style: React.CSSProperties = {
      color: colors.text,
    };

    switch (token.type) {
      case 'keyword':
        style.color = colors.keywords;
        style.fontWeight = 'bold';
        break;
      case 'string':
        style.color = colors.strings;
        break;
      case 'comment':
        style.color = colors.comments;
        style.fontStyle = 'italic';
        break;
      case 'number':
        style.color = colors.numbers;
        break;
      case 'text':
        style.color = colors.text;
        break;
    }

    return (
      <span key={key} style={style}>
        {token.value}
      </span>
    );
  };

  return (
    <div
      className="p-4 rounded font-mono text-sm overflow-x-auto"
      style={{
        backgroundColor: colors.bg,
        color: colors.text,
        ...customStyle,
      }}
    >
      <pre style={{ margin: 0 }}>
        <code>
          {lines.map((line, lineIdx) => (
            <div key={lineIdx} className="flex">
              {showLineNumbers && (
                <span
                  style={{
                    color: colors.lineNumbers,
                    marginRight: '1rem',
                    width: '3rem',
                    textAlign: 'right',
                    userSelect: 'none',
                  }}
                >
                  {lineIdx + 1}
                </span>
              )}
              <span>
                {tokenize(line, language).map((token, tokenIdx) =>
                  renderToken(token, tokenIdx)
                )}
              </span>
            </div>
          ))}
        </code>
      </pre>
    </div>
  );
};

export default SimpleSyntaxHighlighter;
