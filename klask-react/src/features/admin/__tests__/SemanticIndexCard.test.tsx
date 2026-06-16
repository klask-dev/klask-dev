import { render, screen } from '../../../test/utils';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { SemanticIndexCard } from '../components/SemanticIndexCard';
import * as semanticApi from '../../../api/semanticIndex';

vi.mock('react-hot-toast', () => ({
  default: Object.assign(vi.fn(), { success: vi.fn(), error: vi.fn() }),
}));

vi.mock('../../../api/semanticIndex');

const mockedApi = semanticApi as unknown as {
  useSemanticStatus: ReturnType<typeof vi.fn>;
  useStartBackfill: ReturnType<typeof vi.fn>;
  useCancelBackfill: ReturnType<typeof vi.fn>;
};

const mutation = () => ({ mutate: vi.fn(), isPending: false });

function setStatus(data: Partial<semanticApi.SemanticStatusResponse> | undefined) {
  const full = data
    ? {
        enabled: true,
        running: false,
        processed: 0,
        total: null,
        chunks_indexed: 0,
        model: 'mock-model',
        dimension: 384,
        error: null,
        cancelled: false,
        started_at: null,
        finished_at: null,
        ...data,
      }
    : undefined;
  mockedApi.useSemanticStatus.mockReturnValue({ data: full } as never);
}

describe('SemanticIndexCard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedApi.useStartBackfill.mockReturnValue(mutation() as never);
    mockedApi.useCancelBackfill.mockReturnValue(mutation() as never);
  });

  it('renders nothing when semantic search is disabled', () => {
    setStatus({ enabled: false });
    const { container } = render(<SemanticIndexCard />);
    expect(container).toBeEmptyDOMElement();
  });

  it('renders nothing while status is loading', () => {
    setStatus(undefined);
    const { container } = render(<SemanticIndexCard />);
    expect(container).toBeEmptyDOMElement();
  });

  it('shows model, dimension and chunk count when enabled', () => {
    setStatus({ enabled: true, chunks_indexed: 1234, model: 'jina-code', dimension: 768 });
    render(<SemanticIndexCard />);
    expect(screen.getByText('Semantic Index')).toBeInTheDocument();
    // Locale-independent: toLocaleString may insert grouping separators
    // (commas, spaces, NBSP) that vary by environment locale — compare digits.
    expect(
      screen.getByText((content) => content.replace(/\D/g, '') === '1234')
    ).toBeInTheDocument();
    expect(screen.getByText(/jina-code/)).toBeInTheDocument();
    expect(screen.getByText(/768d/)).toBeInTheDocument();
  });

  it('offers "Build Index" when empty and "Rebuild Index" when populated', () => {
    setStatus({ chunks_indexed: 0 });
    const { rerender } = render(<SemanticIndexCard />);
    expect(screen.getByRole('button', { name: /build index/i })).toBeInTheDocument();

    setStatus({ chunks_indexed: 10 });
    rerender(<SemanticIndexCard />);
    expect(screen.getByRole('button', { name: /rebuild index/i })).toBeInTheDocument();
  });

  it('shows a progress bar and cancel button while running', () => {
    setStatus({ running: true, processed: 50, total: 200 });
    render(<SemanticIndexCard />);
    const bar = screen.getByRole('progressbar');
    expect(bar).toHaveAttribute('aria-valuenow', '25');
    expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument();
    expect(screen.getByText(/50 \/ 200 documents/)).toBeInTheDocument();
  });

  it('surfaces the last error after a failed run', () => {
    setStatus({ running: false, error: 'boom' });
    render(<SemanticIndexCard />);
    expect(screen.getByText(/Last rebuild failed: boom/)).toBeInTheDocument();
  });

  it('starts a backfill when the button is clicked', () => {
    const mutate = vi.fn();
    mockedApi.useStartBackfill.mockReturnValue({ mutate, isPending: false } as never);
    setStatus({ chunks_indexed: 0 });
    render(<SemanticIndexCard />);
    screen.getByRole('button', { name: /build index/i }).click();
    expect(mutate).toHaveBeenCalledTimes(1);
  });
});
