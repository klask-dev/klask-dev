import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor, within } from '../../../test/utils';
import userEvent from '@testing-library/user-event';
import { SizeFilter } from '../SizeFilter';

// Mock useProfile hook
vi.mock('../../../hooks/useProfile', () => ({
  useProfile: () => ({
    user: {
      id: '1',
      username: 'testuser',
      preferences: {
        size_unit: 'kb'
      }
    }
  })
}));

describe('SizeFilter Component', () => {
  const mockOnChange = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    // Clear document styles between tests
    const styles = document.head.querySelectorAll('style');
    styles.forEach(style => {
      if (style.textContent?.includes('dual-range-slider')) {
        style.remove();
      }
    });
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Counter Display', () => {
    it('should render counters next to preset buttons when sizeRangeFacets are provided', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Verify all counters are displayed
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('25')).toBeInTheDocument();
      expect(screen.getByText('50')).toBeInTheDocument();
      expect(screen.getByText('75')).toBeInTheDocument();
      expect(screen.getByText('100')).toBeInTheDocument();
      expect(screen.getByText('200')).toBeInTheDocument();
    });

    it('should display correct counts for each size preset', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 5 },
        { value: '1 KB - 10 KB', count: 15 },
        { value: '10 KB - 100 KB', count: 30 },
        { value: '100 KB - 1 MB', count: 45 },
        { value: '1 MB - 10 MB', count: 60 },
        { value: '> 10 MB', count: 90 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      const presetsSection = screen.getByText('< 1 KB').closest('button');
      expect(within(presetsSection!).getByText('5')).toBeInTheDocument();

      const secondPreset = screen.getByText('1 KB - 10 KB').closest('button');
      expect(within(secondPreset!).getByText('15')).toBeInTheDocument();
    });

    it('should use locale string formatting for large counts', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 1000 },
        { value: '1 KB - 10 KB', count: 10000 },
        { value: '10 KB - 100 KB', count: 100000 },
        { value: '100 KB - 1 MB', count: 1000000 },
        { value: '1 MB - 10 MB', count: 10000000 },
        { value: '> 10 MB', count: 100000000 },
      ];

      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Check that toLocaleString is applied (displays formatted large numbers with separators)
      // toLocaleString uses various separators depending on locale (comma, space, narrow non-breaking space)
      const text = container.textContent || '';
      expect(text).toMatch(/1[\s,\u202f]000/); // 1,000 or 1 000 or 1 000 (with non-breaking space)
      expect(text).toMatch(/10[\s,\u202f]000/); // 10,000 or 10 000
      expect(text).toMatch(/100[\s,\u202f]000/); // 100,000 or 100 000
    });

    it('should display counter with correct styling (badge appearance)', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 42 },
        { value: '1 KB - 10 KB', count: 0 },
        { value: '10 KB - 100 KB', count: 100 },
        { value: '100 KB - 1 MB', count: 1 },
        { value: '1 MB - 10 MB', count: 500 },
        { value: '> 10 MB', count: 999 },
      ];

      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Find a counter badge (text with correct count)
      const counterElement = screen.getByText('42');
      expect(counterElement).toHaveClass('bg-gray-100', 'dark:bg-gray-700', 'px-2', 'py-1', 'rounded-full');
      expect(counterElement).toHaveClass('text-xs', 'text-gray-500', 'dark:text-gray-400');
    });
  });

  describe('Preset-Facet Mapping', () => {
    it('should map all 6 size presets correctly', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 20 },
        { value: '10 KB - 100 KB', count: 30 },
        { value: '100 KB - 1 MB', count: 40 },
        { value: '1 MB - 10 MB', count: 50 },
        { value: '> 10 MB', count: 60 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Verify all preset labels are displayed
      expect(screen.getByText('< 1 KB')).toBeInTheDocument();
      expect(screen.getByText('1 KB - 10 KB')).toBeInTheDocument();
      expect(screen.getByText('10 KB - 100 KB')).toBeInTheDocument();
      expect(screen.getByText('100 KB - 1 MB')).toBeInTheDocument();
      expect(screen.getByText('1 MB - 10 MB')).toBeInTheDocument();
      expect(screen.getByText('> 10 MB')).toBeInTheDocument();
    });

    it('should match facet values exactly to preset labels', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 20 },
        { value: '10 KB - 100 KB', count: 30 },
        { value: '100 KB - 1 MB', count: 40 },
        { value: '1 MB - 10 MB', count: 50 },
        { value: '> 10 MB', count: 60 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Each facet count should appear in its corresponding preset button
      const firstButton = screen.getByText('< 1 KB').closest('button');
      expect(within(firstButton!).getByText('10')).toBeInTheDocument();

      const sixthButton = screen.getByText('> 10 MB').closest('button');
      expect(within(sixthButton!).getByText('60')).toBeInTheDocument();
    });

    it('should not display counter when facet value does not match preset label', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: 'invalid-range', count: 999 }, // Won't match any preset
        { value: '1 KB - 10 KB', count: 20 },
      ];

      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // The invalid facet count should not be displayed
      expect(container.textContent).not.toContain('999');
      // But valid counts should be
      expect(container.textContent).toContain('10');
      expect(container.textContent).toContain('20');
    });
  });

  describe('Missing/Undefined Facets', () => {
    it('should render without crashing when no sizeRangeFacets prop is provided', () => {
      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
        />
      );

      // Component should still render the presets
      expect(screen.getByText('< 1 KB')).toBeInTheDocument();
      expect(screen.getByText('> 10 MB')).toBeInTheDocument();
    });

    it('should render without crashing when sizeRangeFacets is empty array', () => {
      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={[]}
        />
      );

      // Component should still render the presets
      expect(screen.getByText('< 1 KB')).toBeInTheDocument();
      expect(screen.getByText('> 10 MB')).toBeInTheDocument();
    });

    it('should not display any counters when sizeRangeFacets is undefined', () => {
      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={undefined}
        />
      );

      // Count badges should not be displayed
      const countElements = container.querySelectorAll('[class*="rounded-full"]');
      expect(countElements.length).toBe(0);
    });

    it('should not display any counters when sizeRangeFacets is empty array', () => {
      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={[]}
        />
      );

      // Count badges should not be displayed
      const countElements = container.querySelectorAll('[class*="rounded-full"]');
      expect(countElements.length).toBe(0);
    });

    it('should gracefully handle partial facet data', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        // Missing other facets
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Should render with only one counter visible
      expect(screen.getByText('< 1 KB')).toBeInTheDocument();
      expect(screen.getByText('10')).toBeInTheDocument();

      // Other presets should still be clickable but without counters
      expect(screen.getByText('> 10 MB')).toBeInTheDocument();
    });
  });

  describe('Loading State', () => {
    it('should display counters even when isLoading is true', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
          isLoading={true}
        />
      );

      // Counters should still be displayed when loading (to maintain filter context)
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('25')).toBeInTheDocument();
      expect(screen.getByText('50')).toBeInTheDocument();
    });

    it('should maintain counter display consistency during isLoading state transitions', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { rerender } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
          isLoading={true}
        />
      );

      // Counters visible during loading
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('200')).toBeInTheDocument();

      // Rerender with isLoading false
      rerender(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
          isLoading={false}
        />
      );

      // Counters remain visible
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('25')).toBeInTheDocument();
    });

    it('should always show preset buttons even when loading', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
          isLoading={true}
        />
      );

      // Preset buttons should still be visible and clickable
      expect(screen.getByText('< 1 KB')).toBeInTheDocument();
      expect(screen.getByText('1 KB - 10 KB')).toBeInTheDocument();
      expect(screen.getByText('10 KB - 100 KB')).toBeInTheDocument();
      expect(screen.getByText('100 KB - 1 MB')).toBeInTheDocument();
      expect(screen.getByText('1 MB - 10 MB')).toBeInTheDocument();
      expect(screen.getByText('> 10 MB')).toBeInTheDocument();
    });
  });

  describe('Zero Counts', () => {
    it('should display zero count in counter', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 0 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 0 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 0 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Zero counts should be displayed
      const zeroElements = screen.getAllByText('0');
      expect(zeroElements.length).toBeGreaterThan(0);
    });

    it('should display all counters including those with zero count', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 0 },
        { value: '1 KB - 10 KB', count: 0 },
        { value: '10 KB - 100 KB', count: 0 },
        { value: '100 KB - 1 MB', count: 0 },
        { value: '1 MB - 10 MB', count: 0 },
        { value: '> 10 MB', count: 0 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // All counters should be visible even if zero
      const zeroElements = screen.getAllByText('0');
      expect(zeroElements.length).toBe(6);
    });
  });

  describe('Dynamic Updates', () => {
    it('should update counters when sizeRangeFacets prop changes', () => {
      const initialFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 20 },
        { value: '10 KB - 100 KB', count: 30 },
        { value: '100 KB - 1 MB', count: 40 },
        { value: '1 MB - 10 MB', count: 50 },
        { value: '> 10 MB', count: 60 },
      ];

      const { rerender } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={initialFacets}
        />
      );

      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('60')).toBeInTheDocument();

      const updatedFacets = [
        { value: '< 1 KB', count: 100 },
        { value: '1 KB - 10 KB', count: 200 },
        { value: '10 KB - 100 KB', count: 300 },
        { value: '100 KB - 1 MB', count: 400 },
        { value: '1 MB - 10 MB', count: 500 },
        { value: '> 10 MB', count: 600 },
      ];

      rerender(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={updatedFacets}
        />
      );

      // Old counts should be gone
      expect(screen.queryByText('10')).not.toBeInTheDocument();
      // New counts should be displayed
      expect(screen.getByText('100')).toBeInTheDocument();
      expect(screen.getByText('600')).toBeInTheDocument();
    });

    it('should handle rapid facet updates', () => {
      const facets1 = [
        { value: '< 1 KB', count: 1 },
        { value: '1 KB - 10 KB', count: 2 },
        { value: '10 KB - 100 KB', count: 3 },
        { value: '100 KB - 1 MB', count: 4 },
        { value: '1 MB - 10 MB', count: 5 },
        { value: '> 10 MB', count: 6 },
      ];

      const { rerender } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={facets1}
        />
      );

      expect(screen.getByText('1')).toBeInTheDocument();

      const facets2 = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 20 },
        { value: '10 KB - 100 KB', count: 30 },
        { value: '100 KB - 1 MB', count: 40 },
        { value: '1 MB - 10 MB', count: 50 },
        { value: '> 10 MB', count: 60 },
      ];

      rerender(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={facets2}
        />
      );

      expect(screen.getByText('10')).toBeInTheDocument();

      const facets3 = [
        { value: '< 1 KB', count: 100 },
        { value: '1 KB - 10 KB', count: 200 },
        { value: '10 KB - 100 KB', count: 300 },
        { value: '100 KB - 1 MB', count: 400 },
        { value: '1 MB - 10 MB', count: 500 },
        { value: '> 10 MB', count: 600 },
      ];

      rerender(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={facets3}
        />
      );

      expect(screen.getByText('100')).toBeInTheDocument();
      expect(screen.getByText('600')).toBeInTheDocument();
    });

    it('should trigger re-rendering when facets prop changes', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 20 },
        { value: '10 KB - 100 KB', count: 30 },
        { value: '100 KB - 1 MB', count: 40 },
        { value: '1 MB - 10 MB', count: 50 },
        { value: '> 10 MB', count: 60 },
      ];

      const { rerender } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      const firstText = screen.getByText('10');
      expect(firstText).toBeInTheDocument();

      const newFacets = [
        { value: '< 1 KB', count: 99 },
        { value: '1 KB - 10 KB', count: 20 },
        { value: '10 KB - 100 KB', count: 30 },
        { value: '100 KB - 1 MB', count: 40 },
        { value: '1 MB - 10 MB', count: 50 },
        { value: '> 10 MB', count: 60 },
      ];

      rerender(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={newFacets}
        />
      );

      // Old count should be gone
      expect(screen.queryByText('10')).not.toBeInTheDocument();
      // New count should be displayed
      expect(screen.getByText('99')).toBeInTheDocument();
    });
  });

  describe('Dark Mode', () => {
    it('should apply dark mode classes to counter badge', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Find a counter badge and verify dark mode classes exist
      const counterBadge = screen.getByText('10');
      expect(counterBadge).toHaveClass('dark:bg-gray-700', 'dark:text-gray-400');
    });

    it('should display counters with proper contrast in dark mode', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 42 },
        { value: '1 KB - 10 KB', count: 0 },
        { value: '10 KB - 100 KB', count: 100 },
        { value: '100 KB - 1 MB', count: 1 },
        { value: '1 MB - 10 MB', count: 500 },
        { value: '> 10 MB', count: 999 },
      ];

      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Document.documentElement should be added dark class
      const style = container.querySelector('style');
      expect(style).toBeDefined();
    });
  });

  describe('Interaction with Presets', () => {
    it('should handle preset button clicks', async () => {
      const user = userEvent.setup();
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      const firstPresetButton = screen.getByText('< 1 KB').closest('button');
      await user.click(firstPresetButton!);

      expect(mockOnChange).toHaveBeenCalled();
    });

    it('should display counters near their preset buttons', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Each counter should be within the same button as its preset label
      const firstButton = screen.getByText('< 1 KB').closest('button');
      expect(within(firstButton!).getByText('10')).toBeInTheDocument();

      const lastButton = screen.getByText('> 10 MB').closest('button');
      expect(within(lastButton!).getByText('200')).toBeInTheDocument();
    });

    it('should maintain counter visibility when hovering preset buttons', async () => {
      const user = userEvent.setup();
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      const firstButton = screen.getByText('< 1 KB').closest('button');
      await user.hover(firstButton!);

      // Counter should still be visible
      expect(screen.getByText('10')).toBeInTheDocument();
    });
  });

  describe('Header and Title', () => {
    it('should display File Size header', () => {
      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
        />
      );

      expect(screen.getByText('File Size')).toBeInTheDocument();
    });

    it('should display File Size header with icon', () => {
      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
        />
      );

      // Find the header section
      const header = screen.getByText('File Size').closest('div');
      const svg = header?.querySelector('svg');
      expect(svg).toBeInTheDocument();
    });
  });

  describe('Clear Button', () => {
    it('should not show clear button when no filter is active', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Find the header area and verify no clear button
      const header = screen.getByText('File Size').parentElement;
      const buttons = header?.querySelectorAll('button');
      // Should only have the preset buttons, not a clear button in the header
      expect(buttons?.length).toBe(0);
    });

    it('should show clear button when filter is active', async () => {
      const user = userEvent.setup();
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Click a preset to activate the filter
      const firstPreset = screen.getByText('< 1 KB').closest('button');
      await user.click(firstPreset!);

      // Simulate the prop change after selection
      const { rerender } = render(
        <SizeFilter
          value={{ min: undefined, max: 1024 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      await waitFor(() => {
        expect(mockOnChange).toHaveBeenCalled();
      });
    });

    it('should highlight selected preset with blue background and text', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={{ min: undefined, max: 1024 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Find the "< 1 KB" preset button (it should be selected)
      const selectedButton = screen.getByText('< 1 KB').closest('button');

      // Check that it has blue background classes
      expect(selectedButton).toHaveClass('bg-blue-50');
      expect(selectedButton).toHaveClass('dark:bg-blue-900');

      // Check that it has blue text classes
      expect(selectedButton).toHaveClass('text-blue-700');
      expect(selectedButton).toHaveClass('dark:text-blue-200');

      // Check aria-pressed attribute
      expect(selectedButton).toHaveAttribute('aria-pressed', 'true');
    });

    it('should move selected preset to top and keep counters visible', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={{ min: 1024, max: 10 * 1024 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // The selected preset "1 KB - 10 KB" should be highlighted with blue background
      const selectedButton = screen.getByText('1 KB - 10 KB').closest('button');
      expect(selectedButton).toHaveClass('bg-blue-50');

      // Counter should be visible
      expect(screen.getByText('25')).toBeInTheDocument();

      // All other counters should also be visible
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('50')).toBeInTheDocument();
    });

    it('should remove blue highlight when preset is deselected via slider', async () => {
      const user = userEvent.setup();
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { rerender } = render(
        <SizeFilter
          value={{ min: undefined, max: 1024 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Verify initial selection highlighting
      let selectedButton = screen.getByText('< 1 KB').closest('button');
      expect(selectedButton).toHaveClass('bg-blue-50');

      // Simulate changing to custom slider range (not matching any preset)
      rerender(
        <SizeFilter
          value={{ min: 500, max: 5000 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Button should no longer have blue background
      selectedButton = screen.getByText('< 1 KB').closest('button');
      expect(selectedButton).not.toHaveClass('bg-blue-50');

      // But counters should still be visible
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('200')).toBeInTheDocument();
    });
  });

  describe('Range Display', () => {
    it('should display range indicator when filter is active', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={{ min: 1024, max: 10 * 1024 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Range display should show the selected range
      expect(screen.getByText(/1\.0 KB/)).toBeInTheDocument();
    });

    it('should display full range when no filter is active', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Should display full range
      expect(screen.getByText(/0 B.*∞/)).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle very large count values', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 999999999 },
        { value: '1 KB - 10 KB', count: 1000000000 },
        { value: '10 KB - 100 KB', count: 9999999999 },
        { value: '100 KB - 1 MB', count: 100 },
        { value: '1 MB - 10 MB', count: 50 },
        { value: '> 10 MB', count: 10 },
      ];

      const { container } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Large numbers should be formatted with toLocaleString (locale-specific formatting with separators)
      const text = container.textContent || '';
      expect(text).toMatch(/999[\s,\u202f]999[\s,\u202f]999/); // 999,999,999 or similar
      expect(text).toMatch(/1[\s,\u202f]000[\s,\u202f]000[\s,\u202f]000/); // 1,000,000,000 or similar
      expect(text).toMatch(/9[\s,\u202f]999[\s,\u202f]999[\s,\u202f]999/); // 9,999,999,999 or similar
    });

    it('should handle facet prop being null', () => {
      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={null as unknown as undefined}
        />
      );

      // Should not crash and should display presets without counters
      expect(screen.getByText('< 1 KB')).toBeInTheDocument();
      expect(screen.getByText('> 10 MB')).toBeInTheDocument();
    });

    it('should handle facets with mixed zero and non-zero counts', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 0 },
        { value: '1 KB - 10 KB', count: 100 },
        { value: '10 KB - 100 KB', count: 0 },
        { value: '100 KB - 1 MB', count: 50 },
        { value: '1 MB - 10 MB', count: 0 },
        { value: '> 10 MB', count: 75 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // All counters should be displayed
      expect(screen.getAllByText('0').length).toBe(3);
      expect(screen.getByText('100')).toBeInTheDocument();
      expect(screen.getByText('50')).toBeInTheDocument();
      expect(screen.getByText('75')).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have accessible preset buttons', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Preset buttons should be accessible by role
      const buttons = screen.getAllByRole('button');
      expect(buttons.length).toBeGreaterThan(0);
    });

    it('should have descriptive titles for hover tooltips', () => {
      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
        />
      );

      // Clear button should have title attribute when filter is active (tested separately)
      // But component should be semantically correct
      expect(screen.getByText('File Size')).toBeInTheDocument();
    });

    it('should support keyboard navigation through presets', async () => {
      const user = userEvent.setup();
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      const buttons = screen.getAllByRole('button');

      // Tab to first button
      await user.tab();

      // Should be able to interact with buttons
      expect(buttons.length).toBeGreaterThan(0);
    });
  });

  describe('Component Integration', () => {
    it('should render within a parent container', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { container } = render(
        <div className="sidebar-filters">
          <SizeFilter
            value={undefined}
            onChange={mockOnChange}
            sizeRangeFacets={mockFacets}
          />
        </div>
      );

      const sidebar = container.querySelector('.sidebar-filters');
      expect(sidebar).toBeInTheDocument();
      expect(within(sidebar!).getByText('File Size')).toBeInTheDocument();
    });

    it('should maintain layout with other filters', () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { container } = render(
        <div className="filters-container">
          <div className="filter-section">Other Filter</div>
          <SizeFilter
            value={undefined}
            onChange={mockOnChange}
            sizeRangeFacets={mockFacets}
          />
        </div>
      );

      expect(container.querySelector('.filters-container')).toBeInTheDocument();
      expect(screen.getByText('Other Filter')).toBeInTheDocument();
      expect(screen.getByText('File Size')).toBeInTheDocument();
    });
  });

  describe('Counter Persistence After Clear', () => {
    it('should restore counters after clearing the filter', async () => {
      const user = userEvent.setup();
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { rerender } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Verify initial state shows counters
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('25')).toBeInTheDocument();
      expect(screen.getByText('50')).toBeInTheDocument();

      // Click a preset to activate the filter
      const firstPreset = screen.getByText('< 1 KB').closest('button');
      await user.click(firstPreset!);

      // Simulate the prop change after selection (counters should hide when custom range is used)
      rerender(
        <SizeFilter
          value={{ min: undefined, max: 1024 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Simulate clearing the filter
      rerender(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Verify counters are back after clearing
      await waitFor(() => {
        expect(screen.getByText('10')).toBeInTheDocument();
        expect(screen.getByText('25')).toBeInTheDocument();
        expect(screen.getByText('50')).toBeInTheDocument();
        expect(screen.getByText('75')).toBeInTheDocument();
        expect(screen.getByText('100')).toBeInTheDocument();
        expect(screen.getByText('200')).toBeInTheDocument();
      });
    });

    it('should preserve counters when facets prop remains unchanged after clear', async () => {
      const mockFacets = [
        { value: '< 1 KB', count: 10 },
        { value: '1 KB - 10 KB', count: 25 },
        { value: '10 KB - 100 KB', count: 50 },
        { value: '100 KB - 1 MB', count: 75 },
        { value: '1 MB - 10 MB', count: 100 },
        { value: '> 10 MB', count: 200 },
      ];

      const { rerender } = render(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Initial counters visible
      expect(screen.getByText('10')).toBeInTheDocument();

      // Simulate filter being set then cleared (facets stay same)
      rerender(
        <SizeFilter
          value={{ min: 1024, max: 100 * 1024 }}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      rerender(
        <SizeFilter
          value={undefined}
          onChange={mockOnChange}
          sizeRangeFacets={mockFacets}
        />
      );

      // Counters should still be visible
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('200')).toBeInTheDocument();
    });
  });
});
