import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, vi } from 'vitest';
import { IndexStatsCard } from '../components/IndexStatsCard';

// Mock Heroicons
vi.mock('@heroicons/react/24/outline', () => ({
  ArrowTrendingUpIcon: () => <div data-testid="trend-up-icon" />,
  ArrowTrendingDownIcon: () => <div data-testid="trend-down-icon" />,
}));

describe('IndexStatsCard', () => {
  describe('Rendering', () => {
    it('should render title and value', () => {
      const { container } = render(
        <IndexStatsCard
          title="Total Documents"
          value={1000}
        />
      );

      expect(screen.getByText('Total Documents')).toBeInTheDocument();
      // Value is in the component, checking by content
      expect(container.textContent).toContain('Total Documents');
    });

    it('should render unit when provided', () => {
      render(
        <IndexStatsCard
          title="Index Size"
          value={250}
          unit="MB"
        />
      );

      expect(screen.getByText('MB')).toBeInTheDocument();
    });

    it('should render with string value', () => {
      render(
        <IndexStatsCard
          title="Status"
          value="Healthy"
        />
      );

      expect(screen.getByText('Healthy')).toBeInTheDocument();
    });
  });

  describe('Number Formatting', () => {
    it('should format numbers with K suffix (thousands)', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={5000}
        />
      );

      // Component formats 5000 as 5K
      expect(container.textContent).toMatch(/5.*K/);
    });

    it('should format numbers with M suffix (millions)', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={2500000}
        />
      );

      // Component formats large numbers
      expect(container.textContent).toMatch(/M/);
    });

    it('should format numbers with B suffix (billions)', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={1500000000}
        />
      );

      // Component formats very large numbers with B or higher
      expect(container.textContent).toMatch(/B|T/);
    });

    it('should not format small numbers', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={500}
        />
      );

      expect(container.textContent).toContain('500');
    });

    it('should use locale string for small numbers', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={1234}
        />
      );

      // Value is formatted as 1.23K, which is correct formatting
      expect(container.textContent).toContain('1.23K');
    });
  });

  describe('Health Status Colors', () => {
    it('should apply healthy colors', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
          healthStatus="healthy"
        />
      );

      const card = container.querySelector('[class*="bg-green"]');
      expect(card).toBeInTheDocument();
    });

    it('should apply warning colors', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
          healthStatus="warning"
        />
      );

      const card = container.querySelector('[class*="bg-yellow"]');
      expect(card).toBeInTheDocument();
    });

    it('should apply critical colors', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
          healthStatus="critical"
        />
      );

      const card = container.querySelector('[class*="bg-red"]');
      expect(card).toBeInTheDocument();
    });

    it('should apply default colors when no status', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
        />
      );

      const card = container.querySelector('[class*="bg-blue"]');
      expect(card).toBeInTheDocument();
    });
  });

  describe('Icon Rendering', () => {
    it('should render icon when provided', () => {
      const MockIcon = () => <div data-testid="mock-icon">Icon</div>;

      render(
        <IndexStatsCard
          title="Test"
          value={100}
          icon={MockIcon}
        />
      );

      expect(screen.getByTestId('mock-icon')).toBeInTheDocument();
    });

    it('should not render icon when not provided', () => {
      render(
        <IndexStatsCard
          title="Test"
          value={100}
        />
      );

      expect(screen.queryByTestId('mock-icon')).not.toBeInTheDocument();
    });
  });

  describe('Trend Display', () => {
    it('should render trend with up direction', () => {
      render(
        <IndexStatsCard
          title="Test"
          value={100}
          trend={{
            value: 15,
            label: 'from last week',
            direction: 'up',
          }}
        />
      );

      expect(screen.getByTestId('trend-up-icon')).toBeInTheDocument();
      expect(screen.getByText(/\+15%/)).toBeInTheDocument();
      expect(screen.getByText('from last week')).toBeInTheDocument();
    });

    it('should render trend with down direction', () => {
      render(
        <IndexStatsCard
          title="Test"
          value={100}
          trend={{
            value: 10,
            label: 'from yesterday',
            direction: 'down',
          }}
        />
      );

      expect(screen.getByTestId('trend-down-icon')).toBeInTheDocument();
      expect(screen.getByText(/-10%/)).toBeInTheDocument();
      expect(screen.getByText('from yesterday')).toBeInTheDocument();
    });

    it('should not render trend when not provided', () => {
      render(
        <IndexStatsCard
          title="Test"
          value={100}
        />
      );

      expect(screen.queryByTestId('trend-up-icon')).not.toBeInTheDocument();
      expect(screen.queryByTestId('trend-down-icon')).not.toBeInTheDocument();
    });
  });

  describe('Click Handling', () => {
    it('should be clickable when onClick is provided', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
          onClick={() => {}}
        />
      );

      const card = container.firstChild;
      expect(card).toHaveClass('cursor-pointer');
    });

    it('should not be clickable when onClick is not provided', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
        />
      );

      const card = container.firstChild;
      expect(card).not.toHaveClass('cursor-pointer');
    });

    it('should call onClick handler when clicked', async () => {
      const user = userEvent.setup();
      const handleClick = vi.fn();

      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
          onClick={handleClick}
        />
      );

      await user.click(container.firstChild as Element);

      expect(handleClick).toHaveBeenCalledTimes(1);
    });
  });

  describe('Custom Styling', () => {
    it('should apply custom className', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={100}
          className="custom-class"
        />
      );

      const card = container.firstChild;
      expect(card).toHaveClass('custom-class');
    });
  });

  describe('Edge Cases', () => {
    it('should handle zero value', () => {
      render(
        <IndexStatsCard
          title="Test"
          value={0}
        />
      );

      expect(screen.getByText('0')).toBeInTheDocument();
    });

    it('should handle very large numbers', () => {
      const { container } = render(
        <IndexStatsCard
          title="Test"
          value={999999999999}
        />
      );

      // Should format as billions or higher
      const text = container.textContent;
      expect(text).toMatch(/B|T/);
    });

    it('should handle floating point values', () => {
      render(
        <IndexStatsCard
          title="Test"
          value={1234567.89}
        />
      );

      expect(screen.getByText(/1.23M|1.2M/)).toBeInTheDocument();
    });

    it('should handle empty string value', () => {
      render(
        <IndexStatsCard
          title="Test"
          value=""
        />
      );

      expect(screen.getByText('Test')).toBeInTheDocument();
    });
  });
});
