import { useState, useEffect } from 'react';

/**
 * Hook to debounce a value with a specified delay
 * @param value The value to debounce
 * @param delay The debounce delay in milliseconds (default: 500ms)
 * @returns The debounced value
 */
export const useDebounce = <T,>(value: T, delay: number = 500): T => {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => clearTimeout(handler);
  }, [value, delay]);

  return debouncedValue;
};
