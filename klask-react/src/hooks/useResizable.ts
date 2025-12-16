import { useState, useRef, useEffect, useCallback } from 'react';

interface UseResizableOptions {
  initialWidth: number;
  minWidth: number;
  maxWidth: number;
  storageKey?: string;
}

export const useResizable = ({
  initialWidth,
  minWidth,
  maxWidth,
  storageKey,
}: UseResizableOptions) => {
  const [width, setWidth] = useState<number>(() => {
    // Try to restore from localStorage if storageKey is provided
    if (storageKey) {
      const stored = localStorage.getItem(storageKey);
      if (stored) {
        const parsedWidth = parseInt(stored, 10);
        if (parsedWidth >= minWidth && parsedWidth <= maxWidth) {
          return parsedWidth;
        }
      }
    }
    return initialWidth;
  });

  const isResizing = useRef(false);
  const startXRef = useRef(0);
  const startWidthRef = useRef(0);

  // Store references to event handlers
  const handleMouseMoveRef = useRef<(e: MouseEvent) => void | null>(null);
  const handleMouseUpRef = useRef<() => void | null>(null);

  // Define handleMouseMove
  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isResizing.current) return;

    const diff = e.clientX - startXRef.current;
    const newWidth = Math.max(minWidth, Math.min(maxWidth, startWidthRef.current + diff));

    setWidth(newWidth);

    // Update localStorage if storageKey is provided
    if (storageKey) {
      localStorage.setItem(storageKey, newWidth.toString());
    }
  }, [minWidth, maxWidth, storageKey]);

  // Define handleMouseUp
  const handleMouseUp = useCallback(() => {
    isResizing.current = false;
    if (handleMouseMoveRef.current) {
      document.removeEventListener('mousemove', handleMouseMoveRef.current);
    }
    if (handleMouseUpRef.current) {
      document.removeEventListener('mouseup', handleMouseUpRef.current);
    }
  }, []);

  // Update refs when handlers change
  useEffect(() => {
    handleMouseMoveRef.current = handleMouseMove;
  }, [handleMouseMove]);

  useEffect(() => {
    handleMouseUpRef.current = handleMouseUp;
  }, [handleMouseUp]);

  const handleMouseDown = useCallback((e: React.MouseEvent<HTMLDivElement>) => {
    isResizing.current = true;
    startXRef.current = e.clientX;
    startWidthRef.current = width;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [handleMouseMove, handleMouseUp, width]);

  useEffect(() => {
    return () => {
      if (handleMouseMoveRef.current) {
        document.removeEventListener('mousemove', handleMouseMoveRef.current);
      }
      if (handleMouseUpRef.current) {
        document.removeEventListener('mouseup', handleMouseUpRef.current);
      }
    };
  }, []);

  return {
    width,
    handleMouseDown,
  };
};
