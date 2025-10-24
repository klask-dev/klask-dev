import React, { useState } from 'react';
import type { SegmentMetrics } from '../../../types/tantivy';

interface SegmentVisualizationProps {
  segments: SegmentMetrics[];
  className?: string;
}

/**
 * Visualizes index segments as horizontal bars
 * Shows document count, size, and deleted docs for each segment
 */
export const SegmentVisualization: React.FC<SegmentVisualizationProps> = ({
  segments,
  className = '',
}) => {
  const [expandedSegmentId, setExpandedSegmentId] = useState<number | null>(null);

  const maxDocs = Math.max(...segments.map(s => s.doc_count), 1);
  const maxSize = Math.max(...segments.map(s => s.size_bytes), 1);

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  };

  const totalDocs = segments.reduce((sum, s) => sum + s.doc_count, 0);
  const totalSize = segments.reduce((sum, s) => sum + s.size_bytes, 0);

  return (
    <div className={`space-y-4 ${className}`}>
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-3">
          <p className="text-xs font-medium text-gray-600">Total Segments</p>
          <p className="text-2xl font-bold text-blue-900 mt-1">
            {segments.length}
          </p>
        </div>
        <div className="bg-purple-50 border border-purple-200 rounded-lg p-3">
          <p className="text-xs font-medium text-gray-600">Total Documents</p>
          <p className="text-2xl font-bold text-purple-900 mt-1">
            {totalDocs.toLocaleString()}
          </p>
        </div>
        <div className="bg-green-50 border border-green-200 rounded-lg p-3">
          <p className="text-xs font-medium text-gray-600">Total Size</p>
          <p className="text-2xl font-bold text-green-900 mt-1">
            {formatBytes(totalSize)}
          </p>
        </div>
      </div>

      <div className="space-y-3">
        {segments.map((segment, idx) => {
          const docRatio = segment.doc_count / maxDocs;
          const sizeRatio = segment.size_bytes / maxSize;
          const hasDeleted = segment.deleted_docs > 0;
          const isExpanded = expandedSegmentId === segment.segment_ord;

          return (
            <div key={segment.segment_ord} className="bg-white border border-gray-200 rounded-lg">
              <button
                onClick={() => setExpandedSegmentId(isExpanded ? null : segment.segment_ord)}
                className="w-full p-4 hover:bg-gray-50 transition-colors"
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-sm font-medium text-gray-900">
                    Segment {segment.segment_ord}
                  </span>
                  <span className="text-xs text-gray-500">
                    ord:{segment.segment_ord} max_doc:{segment.max_doc}
                  </span>
                </div>

                {/* Documents Bar */}
                <div className="mb-2">
                  <div className="flex justify-between text-xs text-gray-600 mb-1">
                    <span>Documents</span>
                    <span>{segment.doc_count.toLocaleString()}</span>
                  </div>
                  <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-blue-500 transition-all"
                      style={{ width: `${docRatio * 100}%` }}
                    />
                  </div>
                </div>

                {/* Size Bar */}
                <div className="mb-2">
                  <div className="flex justify-between text-xs text-gray-600 mb-1">
                    <span>Size</span>
                    <span>{formatBytes(segment.size_bytes)}</span>
                  </div>
                  <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-purple-500 transition-all"
                      style={{ width: `${sizeRatio * 100}%` }}
                    />
                  </div>
                </div>

                {/* Deleted Docs Bar */}
                {hasDeleted && (
                  <div>
                    <div className="flex justify-between text-xs text-gray-600 mb-1">
                      <span>Deleted</span>
                      <span>{segment.deleted_docs.toLocaleString()}</span>
                    </div>
                    <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-red-500 transition-all"
                        style={{ width: `${(segment.deleted_docs / segment.doc_count) * 100}%` }}
                      />
                    </div>
                  </div>
                )}
              </button>

              {/* Expanded Details */}
              {isExpanded && (
                <div className="px-4 pb-4 border-t border-gray-200 bg-gray-50">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <p className="text-gray-600 text-xs font-medium">Segment Ord</p>
                      <p className="text-gray-900 font-mono text-xs mt-1">{segment.segment_ord}</p>
                    </div>
                    <div>
                      <p className="text-gray-600 text-xs font-medium">Max Doc</p>
                      <p className="text-gray-900 text-xs mt-1">{segment.max_doc}</p>
                    </div>
                    <div>
                      <p className="text-gray-600 text-xs font-medium">Size</p>
                      <p className="text-gray-900 font-medium mt-1">{formatBytes(segment.size_bytes)}</p>
                    </div>
                    <div>
                      <p className="text-gray-600 text-xs font-medium">Deleted Docs</p>
                      <p className={`font-medium mt-1 text-xs ${segment.deleted_docs > 0 ? 'text-red-600' : 'text-green-600'}`}>
                        {segment.deleted_docs}
                      </p>
                    </div>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>

      {segments.length === 0 && (
        <div className="text-center py-8 text-gray-500">
          <p>No segments found</p>
        </div>
      )}
    </div>
  );
};
