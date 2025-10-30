import React, { useState, useCallback, useEffect } from 'react';
import { useProfile } from '../../hooks/useProfile';
import { CircleStackIcon } from '@heroicons/react/24/solid';

interface SizeFilterProps {
  value?: { min?: number; max?: number };
  onChange: (value?: { min?: number; max?: number }) => void;
  className?: string;
}

// Logarithmic size configuration
const SIZE_CONFIG = {
  // Log scale: 10^1 to 10^8 bytes (10 bytes to 100MB)
  LOG_MIN: 1, // 10^1 = 10 bytes
  LOG_MAX: 8, // 10^8 = 100 MB
  SLIDER_MIN: 0, // Slider position 0-100
  SLIDER_MAX: 100, // Slider position 0-100
  DEFAULT_MIN_LOG: 2, // 10^2 = 100 bytes
  DEFAULT_MAX_LOG: 6, // 10^6 = 1 MB
} as const;

// Convert slider position (0-100) to log scale (1-8)
const sliderToLog = (sliderValue: number): number => {
  return SIZE_CONFIG.LOG_MIN + (sliderValue / SIZE_CONFIG.SLIDER_MAX) * (SIZE_CONFIG.LOG_MAX - SIZE_CONFIG.LOG_MIN);
};

// Convert log scale (1-8) to slider position (0-100)
const logToSlider = (logValue: number): number => {
  return ((logValue - SIZE_CONFIG.LOG_MIN) / (SIZE_CONFIG.LOG_MAX - SIZE_CONFIG.LOG_MIN)) * SIZE_CONFIG.SLIDER_MAX;
};

// Convert log scale to bytes: 10^logValue
const logToBytes = (logValue: number): number => {
  return Math.pow(10, logValue);
};

// Convert bytes to log scale: log10(bytes)
const bytesToLog = (bytes: number): number => {
  return Math.log10(Math.max(10, bytes)); // Minimum 10 bytes to avoid log(0)
};

// Common file size presets in bytes
const SIZE_PRESETS = [
  { label: '< 1 KB', min: undefined, max: 1024 },
  { label: '1 KB - 100 KB', min: 1024, max: 100 * 1024 },
  { label: '100 KB - 1 MB', min: 100 * 1024, max: 1024 * 1024 },
  { label: '1 MB - 10 MB', min: 1024 * 1024, max: 10 * 1024 * 1024 },
  { label: '> 10 MB', min: 10 * 1024 * 1024, max: undefined },
] as const;

// Helper functions for size conversion
const formatSizeByUnit = (bytes: number, unit: string): string => {
  switch (unit) {
    case 'kb':
      return `${(bytes / 1024).toFixed(1)} KB`;
    case 'mb':
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    case 'bytes':
    default:
      return `${bytes} bytes`;
  }
};

// Smart size formatting - chooses the best unit automatically
const formatSizeSmart = (bytes: number): string => {
  if (bytes >= 1024 * 1024) {
    // >= 1 MB
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  } else if (bytes >= 1024) {
    // >= 1 KB
    return `${(bytes / 1024).toFixed(1)} KB`;
  } else {
    // < 1 KB
    return `${bytes} B`;
  }
};

export const SizeFilter: React.FC<SizeFilterProps> = ({
  value,
  onChange,
  className = '',
}) => {
  // Add custom CSS for slider thumbs pointer events
  React.useEffect(() => {
    const style = document.createElement('style');
    style.textContent = `
      .dual-range-slider input[type="range"]::-webkit-slider-thumb {
        pointer-events: auto !important;
        cursor: pointer;
      }
      .dual-range-slider input[type="range"]::-moz-range-thumb {
        pointer-events: auto !important;
        cursor: pointer;
      }
    `;
    document.head.appendChild(style);

    return () => {
      document.head.removeChild(style);
    };
  }, []);
  const { user } = useProfile();
  const userSizeUnit = user?.preferences?.size_unit || 'kb';

  // Slider values in log scale (0-100 slider positions)
  const [sliderMinPos, setSliderMinPos] = useState<number>(
    value?.min ? logToSlider(bytesToLog(value.min)) : SIZE_CONFIG.SLIDER_MIN
  );
  const [sliderMaxPos, setSliderMaxPos] = useState<number>(
    value?.max ? logToSlider(bytesToLog(value.max)) : SIZE_CONFIG.SLIDER_MAX
  );

  const handleSliderChange = useCallback((newMinPos: number, newMaxPos: number) => {
    // Ensure min <= max
    const actualMinPos = Math.min(newMinPos, newMaxPos);
    const actualMaxPos = Math.max(newMinPos, newMaxPos);

    setSliderMinPos(actualMinPos);
    setSliderMaxPos(actualMaxPos);

    // Convert slider positions to log scale, then to bytes
    const minLogValue = sliderToLog(actualMinPos);
    const maxLogValue = sliderToLog(actualMaxPos);
    const minBytes = Math.round(logToBytes(minLogValue));
    const maxBytes = Math.round(logToBytes(maxLogValue));

    const newValue = {
      min: actualMinPos > SIZE_CONFIG.SLIDER_MIN ? minBytes : undefined,
      max: actualMaxPos < SIZE_CONFIG.SLIDER_MAX ? maxBytes : undefined,
    };

    // Si min et max sont undefined, envoyer undefined pour indiquer "pas de filtre"
    if (newValue.min === undefined && newValue.max === undefined) {
      onChange(undefined);
    } else {
      onChange(newValue);
    }
  }, [onChange]);

  const handlePresetChange = useCallback((preset: { label: string; min?: number; max?: number }) => {
    // Convert preset bytes to slider positions
    const newMinPos = preset.min ? logToSlider(bytesToLog(preset.min)) : SIZE_CONFIG.SLIDER_MIN;
    const newMaxPos = preset.max ? logToSlider(bytesToLog(preset.max)) : SIZE_CONFIG.SLIDER_MAX;

    setSliderMinPos(newMinPos);
    setSliderMaxPos(newMaxPos);

    // Si min et max sont undefined, envoyer undefined pour indiquer "pas de filtre"
    if (preset.min === undefined && preset.max === undefined) {
      onChange(undefined);
    } else {
      onChange({
        min: preset.min,
        max: preset.max,
      });
    }
  }, [onChange]);

  const handleClear = useCallback(() => {
    setSliderMinPos(SIZE_CONFIG.SLIDER_MIN);
    setSliderMaxPos(SIZE_CONFIG.SLIDER_MAX);
    onChange(undefined);
  }, [onChange]);

  // Sync slider positions when value prop changes
  useEffect(() => {
    if (value?.min !== undefined) {
      setSliderMinPos(logToSlider(bytesToLog(value.min)));
    } else {
      setSliderMinPos(SIZE_CONFIG.SLIDER_MIN);
    }

    if (value?.max !== undefined) {
      setSliderMaxPos(logToSlider(bytesToLog(value.max)));
    } else {
      setSliderMaxPos(SIZE_CONFIG.SLIDER_MAX);
    }
  }, [value]);

  const isActive = value && (value.min !== undefined || value.max !== undefined);

  return (
    <div className={`mb-4 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center space-x-2">
          <CircleStackIcon className="h-4 w-4 text-gray-500 dark:text-gray-400" />
          <h4 className="text-xs font-medium text-gray-900 dark:text-white uppercase tracking-wide">File Size</h4>
        </div>
        {isActive && (
          <button
            onClick={handleClear}
            className="text-xs text-gray-400 dark:text-gray-500 hover:text-gray-600 dark:hover:text-gray-300"
            title="Clear filter"
          >
            <svg className="h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        )}
      </div>

      {/* Range Display */}
      <div className="text-xs text-gray-600 dark:text-gray-400 text-center mb-3">
        {isActive ? (() => {
          const minSize = value?.min !== undefined
            ? formatSizeSmart(value.min)
            : '0 B';
          const maxSize = value?.max !== undefined
            ? formatSizeSmart(value.max)
            : '∞';
          return `${minSize} - ${maxSize}`;
        })() : '0 B - ∞'}
      </div>

      {/* Dual Range Slider */}
      <div className="dual-range-slider relative mb-3">
        {/* Track */}
        <div className="h-2 bg-gray-200 dark:bg-gray-600 rounded-lg relative">
          {/* Highlighted Range */}
          <div
            className="absolute h-2 bg-blue-500 rounded-lg"
            style={{
              left: `${sliderMinPos}%`,
              width: `${sliderMaxPos - sliderMinPos}%`
            }}
          />
        </div>

        {/* Min Slider */}
        <input
          type="range"
          min={SIZE_CONFIG.SLIDER_MIN}
          max={SIZE_CONFIG.SLIDER_MAX}
          step="1"
          value={sliderMinPos}
          onChange={(e) => handleSliderChange(parseInt(e.target.value), sliderMaxPos)}
          className="absolute top-0 w-full h-2 bg-transparent appearance-none cursor-pointer"
          style={{
            zIndex: 1,
            background: 'transparent',
            outline: 'none',
            pointerEvents: 'none'
          }}
        />

        {/* Max Slider */}
        <input
          type="range"
          min={SIZE_CONFIG.SLIDER_MIN}
          max={SIZE_CONFIG.SLIDER_MAX}
          step="1"
          value={sliderMaxPos}
          onChange={(e) => handleSliderChange(sliderMinPos, parseInt(e.target.value))}
          className="absolute top-0 w-full h-2 bg-transparent appearance-none cursor-pointer"
          style={{
            zIndex: 1,
            background: 'transparent',
            outline: 'none',
            pointerEvents: 'none'
          }}
        />
      </div>

      {/* Quick Size Buttons (Presets) */}
      <div className="space-y-1">
        {SIZE_PRESETS.map((preset, index) => (
          <button
            key={index}
            onClick={() => handlePresetChange(preset)}
            className="flex items-center justify-between px-2 py-1 rounded cursor-pointer transition-colors text-sm w-full text-left hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300"
          >
            <span className="truncate text-xs min-w-0 flex-1" title={preset.label}>
              {preset.label}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};