import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import TokenDisplay from '../TokenDisplay';
import type { CreateTokenResponse } from '../../../../types';

describe('TokenDisplay', () => {
  const mockToken: CreateTokenResponse = {
    id: '1',
    token: 'klask_pat_test123456789abcdefgh',
    token_prefix: 'klask_pat_test123',
    name: 'Test Token',
    created_at: '2026-06-15T00:00:00Z',
  };

  const mockOnDone = vi.fn();

  it('should render token display section', () => {
    render(<TokenDisplay token={mockToken} onDone={mockOnDone} />);

    expect(screen.getByText('Token Created Successfully')).toBeInTheDocument();
    expect(screen.getByText('Your API Token')).toBeInTheDocument();
    expect(screen.getByText(/Keep this token safe/)).toBeInTheDocument();
  });

  it('should display the token in a code block', () => {
    render(<TokenDisplay token={mockToken} onDone={mockOnDone} />);

    expect(screen.getByText(mockToken.token)).toBeInTheDocument();
  });

  it('should display the token prefix', () => {
    render(<TokenDisplay token={mockToken} onDone={mockOnDone} />);

    expect(screen.getByText(`${mockToken.token_prefix}...`)).toBeInTheDocument();
  });

  it('should have a copy button', () => {
    render(<TokenDisplay token={mockToken} onDone={mockOnDone} />);

    const copyButton = screen.getByRole('button', { name: /Copy/i });
    expect(copyButton).toBeInTheDocument();
  });

  it('should call onDone when Done button is clicked', async () => {
    const user = userEvent.setup();
    render(<TokenDisplay token={mockToken} onDone={mockOnDone} />);

    const doneButton = screen.getByRole('button', { name: /Done/i });
    await user.click(doneButton);

    expect(mockOnDone).toHaveBeenCalled();
  });

  it('should display warning about token safety', () => {
    render(<TokenDisplay token={mockToken} onDone={mockOnDone} />);

    expect(screen.getByText(/Do not commit this token to git/i)).toBeInTheDocument();
  });
});
