import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '../../../test/utils';
import userEvent from '@testing-library/user-event';
import { SearchBar } from '../SearchBar';

describe('SearchBar Component', () => {
  const mockOnChange = vi.fn();
  const mockOnSearch = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  const defaultProps = {
    value: '',
    onChange: mockOnChange,
    onSearch: mockOnSearch,
  };

  it('should render with default placeholder', () => {
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByPlaceholderText('Search in your codebase...');
    expect(input).toBeInTheDocument();
  });

  it('should render with custom placeholder', () => {
    const customPlaceholder = 'Custom search placeholder';
    render(<SearchBar {...defaultProps} placeholder={customPlaceholder} />);
    
    const input = screen.getByPlaceholderText(customPlaceholder);
    expect(input).toBeInTheDocument();
  });

  it('should display the provided value', () => {
    render(<SearchBar {...defaultProps} value="test query" />);
    
    const input = screen.getByDisplayValue('test query');
    expect(input).toBeInTheDocument();
  });

  it('should call onChange when user types', async () => {
    render(<SearchBar {...defaultProps} />);
    const input = screen.getByRole('textbox');

    // Use fireEvent instead of userEvent for speed
    fireEvent.change(input, { target: { value: 'test' } });

    // Should not call immediately
    expect(mockOnChange).not.toHaveBeenCalled();

    // Wait for debounce (300ms)
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith('test');
    }, { timeout: 500 });
  });

  it('should debounce onChange and onSearch calls', async () => {
    render(<SearchBar {...defaultProps} />);
    const input = screen.getByRole('textbox');

    // Type multiple times quickly (simulating rapid typing)
    fireEvent.change(input, { target: { value: 't' } });
    fireEvent.change(input, { target: { value: 'te' } });
    fireEvent.change(input, { target: { value: 'tes' } });
    fireEvent.change(input, { target: { value: 'test' } });

    // Should not call immediately
    expect(mockOnChange).not.toHaveBeenCalled();

    // Wait for debounce and check only called once with final value
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledTimes(1);
      expect(mockOnChange).toHaveBeenCalledWith('test');
      expect(mockOnSearch).toHaveBeenCalledTimes(1);
      expect(mockOnSearch).toHaveBeenCalledWith('test');
    }, { timeout: 500 });
  });

  it('should call onSearch when form is submitted', () => {
    render(<SearchBar {...defaultProps} />);

    const input = screen.getByRole('textbox');
    const form = input.closest('form');

    // Use fireEvent instead of userEvent for speed
    fireEvent.change(input, { target: { value: 'submit test' } });
    fireEvent.submit(form!);

    expect(mockOnSearch).toHaveBeenCalledWith('submit test');
  });

  it('should show loading state', () => {
    const { container } = render(<SearchBar {...defaultProps} isLoading={true} />);

    const loadingIcon = container.querySelector('svg');
    expect(loadingIcon).toHaveClass('animate-pulse', 'text-primary-500');
  });

  it('should show normal search icon when not loading', () => {
    const { container } = render(<SearchBar {...defaultProps} isLoading={false} />);

    const searchIcon = container.querySelector('svg');
    expect(searchIcon).toHaveClass('text-gray-400');
    expect(searchIcon).not.toHaveClass('animate-pulse');
  });

  it('should show clear button when there is text', () => {
    render(<SearchBar {...defaultProps} value="test" />);

    const clearButton = screen.getByRole('button');
    expect(clearButton).toBeInTheDocument();
  });

  it('should not show clear button when there is no text', () => {
    render(<SearchBar {...defaultProps} value="" />);
    
    const clearButton = screen.queryByRole('button');
    expect(clearButton).not.toBeInTheDocument();
  });

  it('should clear input when clear button is clicked', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByRole('textbox');
    await user.type(input, 'test');
    
    const clearButton = screen.getByRole('button');
    await user.click(clearButton);
    
    expect(mockOnChange).toHaveBeenCalledWith('');
    expect(mockOnSearch).toHaveBeenCalledWith('');
  });

  it('should sync local value with prop value', () => {
    const { rerender } = render(<SearchBar {...defaultProps} value="initial" />);
    
    const input = screen.getByDisplayValue('initial');
    expect(input).toBeInTheDocument();
    
    rerender(<SearchBar {...defaultProps} value="updated" />);
    
    const updatedInput = screen.getByDisplayValue('updated');
    expect(updatedInput).toBeInTheDocument();
  });

  it('should prevent default on form submission', () => {
    render(<SearchBar {...defaultProps} />);
    
    const form = screen.getByRole('textbox').closest('form');
    const submitEvent = new Event('submit', { bubbles: true, cancelable: true });
    
    form!.dispatchEvent(submitEvent);
    
    expect(submitEvent.defaultPrevented).toBe(true);
  });

  it('should have correct input attributes', () => {
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('type', 'text');
    expect(input).toHaveAttribute('autoComplete', 'off');
    expect(input).toHaveAttribute('spellCheck', 'false');
  });

  it('should have correct CSS classes', () => {
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByRole('textbox');
    expect(input).toHaveClass(
      'block', 'w-full', 'pl-10', 'pr-12', 'py-3', 'text-lg',
      'border', 'border-gray-300', 'rounded-lg',
      'focus:ring-2', 'focus:ring-blue-500', 'focus:border-blue-500',
      'placeholder-gray-400'
    );
  });

  it('should handle rapid typing correctly', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByRole('textbox');
    
    // Type rapidly
    await user.type(input, 'a');
    await user.type(input, 'b');
    await user.type(input, 'c');
    
    // Wait for debounce
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith('abc');
    }, { timeout: 500 });
  });

  it('should not call onChange/onSearch if debounced value equals current value', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} value="test" />);
    
    const input = screen.getByRole('textbox');
    
    // Clear and type the same value
    await user.clear(input);
    await user.type(input, 'test');
    
    // Wait for debounce - should not call since value is the same
    await new Promise(resolve => setTimeout(resolve, 400));
    
    expect(mockOnChange).not.toHaveBeenCalled();
    expect(mockOnSearch).not.toHaveBeenCalled();
  });

  it('should handle empty string correctly', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} value="test" />);
    
    const input = screen.getByRole('textbox');
    await user.clear(input);
    
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith('');
    }, { timeout: 500 });
  });

  it('should clear input when clear button is used', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} />);

    const input = screen.getByRole('textbox');
    await user.type(input, 'test');

    // Verify text is there
    expect(input).toHaveValue('test');

    const clearButton = screen.getByRole('button');
    await user.click(clearButton);

    // Verify input is cleared and onChange was called
    expect(input).toHaveValue('');
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith('');
    }, { timeout: 500 });
  });

  it('should handle special characters correctly', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByRole('textbox');
    const specialText = 'test@#$%^&*()[]{}';

    // Use paste instead of type for special characters to avoid parsing issues
    await user.click(input);
    await user.paste(specialText);
    
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith(specialText);
    }, { timeout: 500 });
  });

  it('should handle unicode characters correctly', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByRole('textbox');
    const unicodeText = 'café naïve résumé';
    
    await user.type(input, unicodeText);
    
    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith(unicodeText);
    }, { timeout: 500 });
  });

  it('should handle very long text correctly', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} />);

    const input = screen.getByRole('textbox');
    const longText = 'a'.repeat(1000);

    // Use paste instead of type for long text to avoid timeout
    await user.click(input);
    await user.paste(longText);

    await waitFor(() => {
      expect(mockOnChange).toHaveBeenCalledWith(longText);
    }, { timeout: 500 });
  });

  it('should be accessible', () => {
    render(<SearchBar {...defaultProps} />);
    
    const input = screen.getByRole('textbox');
    expect(input).toBeInTheDocument();
    
    // Should be able to find by placeholder text (accessibility)
    expect(screen.getByPlaceholderText('Search in your codebase...')).toBeInTheDocument();
  });

  it('should support keyboard navigation', async () => {
    const user = userEvent.setup();
    render(<SearchBar {...defaultProps} />);

    const input = screen.getByRole('textbox');

    // Tab should focus the input
    await user.tab();
    expect(input).toHaveFocus();

    // Type text and wait for it to be in the input
    await user.type(input, 'test');
    expect(input).toHaveValue('test');

    // Enter should submit the form with the current input value
    await user.keyboard('{Enter}');

    expect(mockOnSearch).toHaveBeenCalledWith('test');
  });
});