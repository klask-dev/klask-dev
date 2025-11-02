import React, { useState, useEffect, useCallback } from 'react';
import { useProfile } from '../../hooks/useProfile';

interface SizeRangeFilterProps {
  value?: { min?: number; max?: number };
  onChange: (value?: { min?: number; max?: number }) => void;
  className?: string;
}

// Size range configuration (in bytes)
const SIZE_CONFIG = {
  MIN: 0,
  MAX: 100 * 1024 * 1024, // 100 MB max
  STEP: 1024, // 1 KB step
  DEFAULT_MIN: 0,
  DEFAULT_MAX: 10 * 1024 * 1024, // 10 MB default max
} as const;

// Common file size presets in bytes
const SIZE_PRESETS = [
  { label: '< 1 KB', min: undefined, max: 1024 },
  { label: '1 KB - 10 KB', min: 1024, max: 10 * 1024 },
  { label: '10 KB - 100 KB', min: 10 * 1024, max: 100 * 1024 },
  { label: '100 KB - 1 MB', min: 100 * 1024, max: 1024 * 1024 },
  { label: '> 1 MB', min: 1024 * 1024, max: undefined },
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

const convertToBytes = (value: number, unit: string): number => {
  switch (unit) {
    case 'kb':
      return value * 1024;
    case 'mb':
      return value * 1024 * 1024;
    case 'bytes':
    default:
      return value;
  }
};

const convertFromBytes = (bytes: number, unit: string): number => {
  switch (unit) {
    case 'kb':
      return bytes / 1024;
    case 'mb':
      return bytes / (1024 * 1024);
    case 'bytes':
    default:
      return bytes;
  }
};

export const SizeRangeFilter: React.FC<SizeRangeFilterProps> = ({
  value,
  onChange,
  className = '',
}) => {
  const { user } = useProfile();
  const userSizeUnit = user?.preferences?.size_unit || 'kb';

  const [mode, setMode] = useState<'preset' | 'slider' | 'custom'>('preset');
  const [selectedPreset, setSelectedPreset] = useState<number | null>(null);
  const [sliderMin, setSliderMin] = useState<number>(SIZE_CONFIG.DEFAULT_MIN);
  const [sliderMax, setSliderMax] = useState<number>(SIZE_CONFIG.DEFAULT_MAX);
  const [customMin, setCustomMin] = useState<string>('');
  const [customMax, setCustomMax] = useState<string>('');

  // Initialize state from value prop
  useEffect(() => {
    if (!value || (!value.min && !value.max)) {
      setMode('preset');
      setSelectedPreset(null);
      setSliderMin(SIZE_CONFIG.DEFAULT_MIN);
      setSliderMax(SIZE_CONFIG.DEFAULT_MAX);
      setCustomMin('');
      setCustomMax('');
      return;
    }

    // Check if current value matches a preset
    const presetIndex = SIZE_PRESETS.findIndex(preset =>
      preset.min === value.min && preset.max === value.max
    );

    if (presetIndex >= 0) {
      setMode('preset');
      setSelectedPreset(presetIndex);
    } else {
      // Set slider values
      setSliderMin(value.min || SIZE_CONFIG.MIN);
      setSliderMax(value.max || SIZE_CONFIG.MAX);

      // If it's a simple range, use slider, otherwise custom
      const isSimpleRange = (value.min || SIZE_CONFIG.MIN) >= SIZE_CONFIG.MIN &&
                           (value.max || SIZE_CONFIG.MAX) <= SIZE_CONFIG.MAX;

      if (isSimpleRange) {
        setMode('slider');
      } else {
        setMode('custom');
        setCustomMin(value.min ? convertFromBytes(value.min, userSizeUnit).toString() : '');
        setCustomMax(value.max ? convertFromBytes(value.max, userSizeUnit).toString() : '');
      }
    }
  }, [value, userSizeUnit]);

  const handlePresetChange = useCallback((index: number) => {
    setSelectedPreset(index);
    const preset = SIZE_PRESETS[index];
    onChange({
      min: preset.min,
      max: preset.max,
    });
  }, [onChange]);

  const handleSliderChange = useCallback((newMin: number, newMax: number) => {
    // Ensure min <= max
    const actualMin = Math.min(newMin, newMax);
    const actualMax = Math.max(newMin, newMax);

    setSliderMin(actualMin);
    setSliderMax(actualMax);

    onChange({
      min: actualMin > SIZE_CONFIG.MIN ? actualMin : undefined,
      max: actualMax < SIZE_CONFIG.MAX ? actualMax : undefined,
    });
  }, [onChange]);

  const handleCustomChange = useCallback(() => {
    const minBytes = customMin ? convertToBytes(parseFloat(customMin), userSizeUnit) : undefined;
    const maxBytes = customMax ? convertToBytes(parseFloat(customMax), userSizeUnit) : undefined;

    if (!minBytes && !maxBytes) {
      onChange(undefined);
    } else {
      onChange({
        min: minBytes,
        max: maxBytes,
      });
    }
  }, [customMin, customMax, userSizeUnit, onChange]);

  const handleClear = useCallback(() => {
    setSelectedPreset(null);
    setSliderMin(SIZE_CONFIG.DEFAULT_MIN);
    setSliderMax(SIZE_CONFIG.DEFAULT_MAX);
    setCustomMin('');
    setCustomMax('');
    onChange(undefined);
  }, [onChange]);

  const isActive = value && (value.min !== undefined || value.max !== undefined);

  return (
    <div className={`space-y-3 ${className}`}>
        <div className="flex items-center justify-between">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            File Size
          </label>
          {isActive && (
            <button
              onClick={handleClear}
              className="text-xs text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
            >
              Clear
            </button>
          )}
        </div>

      {/* Mode Toggle */}
      <div className="flex gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
        <button
          onClick={() => setMode('preset')}
          className={`flex-1 px-2 py-1.5 text-xs font-medium rounded-md transition-colors ${
            mode === 'preset'
              ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-gray-100 shadow-sm'
              : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
          }`}
        >
          Presets
        </button>
        <button
          onClick={() => setMode('slider')}
          className={`flex-1 px-2 py-1.5 text-xs font-medium rounded-md transition-colors ${
            mode === 'slider'
              ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-gray-100 shadow-sm'
              : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
          }`}
        >
          Slider
        </button>
        <button
          onClick={() => setMode('custom')}
          className={`flex-1 px-2 py-1.5 text-xs font-medium rounded-md transition-colors ${
            mode === 'custom'
              ? 'bg-white dark:bg-gray-600 text-gray-900 dark:text-gray-100 shadow-sm'
              : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
          }`}
        >
          Custom
        </button>
      </div>

      {mode === 'preset' && (
        <div className="space-y-2">
          {SIZE_PRESETS.map((preset, index) => (
            <button
              key={index}
              onClick={() => handlePresetChange(index)}
              className={`w-full text-left px-3 py-2 text-sm rounded-lg border transition-colors ${
                selectedPreset === index
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-900 dark:text-blue-100'
                  : 'border-gray-200 dark:border-gray-600 hover:border-gray-300 dark:hover:border-gray-500 hover:bg-gray-50 dark:hover:bg-gray-700/50'
              }`}
            >
              {preset.label}
            </button>
          ))}
        </div>
      )}

      {mode === 'slider' && (
        <div className="space-y-4">
          {/* Range Display */}
          <div className="text-xs text-gray-600 dark:text-gray-400 text-center">
            {formatSizeByUnit(sliderMin, userSizeUnit)} - {formatSizeByUnit(sliderMax, userSizeUnit)}
          </div>

          {/* Dual Range Slider */}
          <div className="relative">
            {/* Track */}
            <div className="h-2 bg-gray-200 dark:bg-gray-600 rounded-lg relative">
              {/* Highlighted Range */}
              <div
                className="absolute h-2 bg-blue-500 rounded-lg"
                style={{
                  left: `${((sliderMin - SIZE_CONFIG.MIN) / (SIZE_CONFIG.MAX - SIZE_CONFIG.MIN)) * 100}%`,
                  width: `${((sliderMax - sliderMin) / (SIZE_CONFIG.MAX - SIZE_CONFIG.MIN)) * 100}%`
                }}
              />
            </div>

            {/* Min Slider */}
            <input
              type="range"
              min={SIZE_CONFIG.MIN}
              max={SIZE_CONFIG.MAX}
              step={SIZE_CONFIG.STEP}
              value={sliderMin}
              onChange={(e) => handleSliderChange(parseInt(e.target.value), sliderMax)}
              className="absolute top-0 w-full h-2 bg-transparent appearance-none cursor-pointer"
              style={{
                zIndex: sliderMin > sliderMax - SIZE_CONFIG.STEP * 10 ? 2 : 1,
                background: 'transparent',
                outline: 'none'
              }}
            />

            {/* Max Slider */}
            <input
              type="range"
              min={SIZE_CONFIG.MIN}
              max={SIZE_CONFIG.MAX}
              step={SIZE_CONFIG.STEP}
              value={sliderMax}
              onChange={(e) => handleSliderChange(sliderMin, parseInt(e.target.value))}
              className="absolute top-0 w-full h-2 bg-transparent appearance-none cursor-pointer"
              style={{
                zIndex: sliderMax < sliderMin + SIZE_CONFIG.STEP * 10 ? 2 : 1,
                background: 'transparent',
                outline: 'none'
              }}
            />
          </div>

          {/* Quick Size Buttons */}
          <div className="grid grid-cols-3 gap-2">
            <button
              onClick={() => handleSliderChange(0, 1024)}
              className="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              ≤ 1 KB
            </button>
            <button
              onClick={() => handleSliderChange(1024, 1024 * 1024)}
              className="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              1 KB - 1 MB
            </button>
            <button
              onClick={() => handleSliderChange(1024 * 1024, SIZE_CONFIG.MAX)}
              className="px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded hover:bg-gray-50 dark:hover:bg-gray-700"
            >
              ≥ 1 MB
            </button>
          </div>
        </div>
      )}

      {mode === 'custom' && (
        <div className="space-y-3">
          <div>
            <label className="block text-xs text-gray-600 dark:text-gray-400 mb-1">
              Min Size ({userSizeUnit.toUpperCase()})
            </label>
            <input
              type="number"
              min="0"
              step="0.1"
              value={customMin}
              onChange={(e) => setCustomMin(e.target.value)}
              onBlur={handleCustomChange}
              className="w-full px-3 py-2 text-sm border border-gray-200 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="No minimum"
            />
          </div>

          <div>
            <label className="block text-xs text-gray-600 dark:text-gray-400 mb-1">
              Max Size ({userSizeUnit.toUpperCase()})
            </label>
            <input
              type="number"
              min="0"
              step="0.1"
              value={customMax}
              onChange={(e) => setCustomMax(e.target.value)}
              onBlur={handleCustomChange}
              className="w-full px-3 py-2 text-sm border border-gray-200 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              placeholder="No maximum"
            />
          </div>

          {(customMin || customMax) && (
            <div className="text-xs text-gray-500 dark:text-gray-400">
              {customMin && `Min: ${formatSizeByUnit(convertToBytes(parseFloat(customMin), userSizeUnit), userSizeUnit)}`}
              {customMin && customMax && ' • '}
              {customMax && `Max: ${formatSizeByUnit(convertToBytes(parseFloat(customMax), userSizeUnit), userSizeUnit)}`}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
