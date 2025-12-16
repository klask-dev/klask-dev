import React from 'react';

interface ResizeHandleProps {
  onMouseDown: (e: React.MouseEvent<HTMLDivElement>) => void;
}

export const ResizeHandle: React.FC<ResizeHandleProps> = ({ onMouseDown }) => {
  return (
    <div
      onMouseDown={onMouseDown}
      className="group absolute right-0 top-0 bottom-0 cursor-col-resize select-none"
      style={{
        userSelect: 'none',
        width: '6px',
        right: '-3px',
      }}
    >
      {/* Visual indicator - thin line in the center */}
      <div className="absolute left-1/2 top-0 bottom-0 w-px bg-gray-300 dark:bg-gray-600 -translate-x-1/2 group-hover:bg-blue-400 dark:group-hover:bg-blue-400 transition-colors" />
    </div>
  );
};
