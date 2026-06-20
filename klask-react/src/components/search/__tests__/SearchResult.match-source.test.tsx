import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SearchResult } from '../SearchResult';
import type { SearchResult as SearchResultType, MatchSource } from '../../../types';

const baseResult: SearchResultType = {
  file_id: '1',
  doc_address: '0:1',
  name: 'auth.rs',
  path: 'src/auth.rs',
  content_snippet: 'fn login() {}',
  project: 'klask',
  version: 'main',
  extension: 'rs',
  score: 0.9,
};

function renderWith(match_source?: MatchSource) {
  return render(
    <SearchResult result={{ ...baseResult, match_source }} query="login" onFileClick={vi.fn()} />
  );
}

describe('SearchResult - match source badge', () => {
  it('renders no badge when match_source is absent (keyword path)', () => {
    renderWith(undefined);
    expect(screen.queryByText('Keyword')).not.toBeInTheDocument();
    expect(screen.queryByText('Semantic')).not.toBeInTheDocument();
    expect(screen.queryByText('Both')).not.toBeInTheDocument();
  });

  it('renders the Semantic badge', () => {
    renderWith('semantic');
    expect(screen.getByText('Semantic')).toBeInTheDocument();
  });

  it('renders the Both badge', () => {
    renderWith('both');
    expect(screen.getByText('Both')).toBeInTheDocument();
  });

  it('renders the Keyword badge', () => {
    renderWith('keyword');
    expect(screen.getByText('Keyword')).toBeInTheDocument();
  });
});
