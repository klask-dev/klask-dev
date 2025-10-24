import React, { useState } from 'react';
import { SparklesIcon } from '@heroicons/react/24/outline';
import type { TuningSettingsResponse, TuningRecommendation } from '../../../types/tantivy';

interface TuningPanelProps {
  tuning: TuningSettingsResponse;
  onOptimize?: () => void;
  isOptimizing?: boolean;
  className?: string;
}

/**
 * Displays tuning recommendations for index optimization
 */
export const TuningPanel: React.FC<TuningPanelProps> = ({
  tuning,
  onOptimize,
  isOptimizing = false,
  className = '',
}) => {
  const [expandedRec, setExpandedRec] = useState<number | null>(null);

  const getImpactColor = (impact: string) => {
    switch (impact) {
      case 'High':
        return 'bg-red-50 border-red-200';
      case 'Medium':
        return 'bg-yellow-50 border-yellow-200';
      case 'Low':
      default:
        return 'bg-blue-50 border-blue-200';
    }
  };

  const getImpactBadgeColor = (impact: string) => {
    switch (impact) {
      case 'High':
        return 'bg-red-100 text-red-800';
      case 'Medium':
        return 'bg-yellow-100 text-yellow-800';
      case 'Low':
      default:
        return 'bg-blue-100 text-blue-800';
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Summary */}
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
        <p className="text-sm text-gray-900">{tuning.summary}</p>
        <p className="text-xs text-gray-500 mt-2">
          Analyzed at: {new Date(tuning.analyzed_at).toLocaleString()}
        </p>
      </div>

      {/* Recommendations */}
      {tuning.recommendations.length > 0 && (
        <div className="space-y-3">
          <h3 className="text-sm font-semibold text-gray-900">
            Recommendations ({tuning.recommendations.length})
          </h3>
          <div className="space-y-2">
            {tuning.recommendations.map((rec: TuningRecommendation, idx) => (
              <div
                key={idx}
                className={`${getImpactColor(rec.impact)} border rounded-lg overflow-hidden`}
              >
                <button
                  onClick={() => setExpandedRec(expandedRec === idx ? null : idx)}
                  className="w-full px-4 py-3 flex items-start justify-between hover:bg-opacity-75 transition-colors"
                >
                  <div className="text-left flex-1">
                    <div className="flex items-center gap-2">
                      <h4 className="text-sm font-semibold text-gray-900">
                        {rec.title}
                      </h4>
                      <span
                        className={`${getImpactBadgeColor(
                          rec.impact
                        )} text-xs font-semibold px-2 py-1 rounded`}
                      >
                        {rec.impact}
                      </span>
                    </div>
                    <p className="text-xs text-gray-600 mt-1">
                      {rec.description}
                    </p>
                  </div>
                  <div
                    className={`transform transition-transform ${
                      expandedRec === idx ? 'rotate-180' : ''
                    }`}
                  >
                    <span className="text-gray-400">▼</span>
                  </div>
                </button>

                {/* Expanded Details */}
                {expandedRec === idx && (
                  <div className="px-4 py-3 border-t border-opacity-20 border-current bg-opacity-30 bg-current">
                    <div className="space-y-3">
                      <div>
                        <p className="text-xs font-medium text-gray-600">
                          Reason
                        </p>
                        <p className="text-sm text-gray-900 mt-1">
                          {rec.reason}
                        </p>
                      </div>

                      {rec.parameter && (
                        <div>
                          <p className="text-xs font-medium text-gray-600">
                            Parameter
                          </p>
                          <p className="text-sm font-mono text-gray-900 mt-1">
                            {rec.parameter}
                          </p>
                        </div>
                      )}

                      {rec.current_value && (
                        <div>
                          <p className="text-xs font-medium text-gray-600">
                            Current Value
                          </p>
                          <p className="text-sm font-mono text-gray-900 mt-1">
                            {rec.current_value}
                          </p>
                        </div>
                      )}

                      {rec.recommended_value && (
                        <div>
                          <p className="text-xs font-medium text-gray-600">
                            Recommended Value
                          </p>
                          <p className="text-sm font-mono text-blue-600 mt-1">
                            {rec.recommended_value}
                          </p>
                        </div>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* No Recommendations State */}
      {tuning.recommendations.length === 0 && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4 text-center">
          <p className="text-sm font-medium text-green-900">
            No tuning recommendations
          </p>
          <p className="text-xs text-green-700 mt-1">
            Your index is well-optimized
          </p>
        </div>
      )}

      {/* Optimize Button */}
      {onOptimize && (
        <button
          onClick={onOptimize}
          disabled={isOptimizing}
          className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-medium rounded-lg transition-colors"
        >
          <SparklesIcon className="h-5 w-5" />
          {isOptimizing ? 'Optimizing...' : 'Optimize Index'}
        </button>
      )}
    </div>
  );
};
