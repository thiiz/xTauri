import React, { useEffect, useState } from 'react';
import { useProfileStore } from '../stores/profileStore';
import { useXtreamContentStore } from '../stores/xtreamContentStore';
import { XtreamHistory } from '../types/types';
import { TvIcon } from './Icons';

interface HistoryViewProps {
  onPlayContent?: (history: XtreamHistory) => void;
}

// Custom icon components
const ClockIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <circle cx="12" cy="12" r="10" />
    <polyline points="12 6 12 12 16 14" />
  </svg>
);

const FilmIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
    <line x1="7" y1="2" x2="7" y2="22" />
    <line x1="17" y1="2" x2="17" y2="22" />
    <line x1="2" y1="12" x2="22" y2="12" />
    <line x1="2" y1="7" x2="7" y2="7" />
    <line x1="2" y1="17" x2="7" y2="17" />
    <line x1="17" y1="17" x2="22" y2="17" />
    <line x1="17" y1="7" x2="22" y2="7" />
  </svg>
);

const RadioIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <circle cx="12" cy="12" r="2" />
    <path d="M16.24 7.76a6 6 0 0 1 0 8.49m-8.48-.01a6 6 0 0 1 0-8.49m11.31-2.82a10 10 0 0 1 0 14.14m-14.14 0a10 10 0 0 1 0-14.14" />
  </svg>
);

const Trash2Icon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <polyline points="3 6 5 6 21 6" />
    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    <line x1="10" y1="11" x2="10" y2="17" />
    <line x1="14" y1="11" x2="14" y2="17" />
  </svg>
);

const PlayIconSmall = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <polygon points="5 3 19 12 5 21 5 3" />
  </svg>
);

export const HistoryView: React.FC<HistoryViewProps> = ({ onPlayContent }) => {
  const { activeProfile } = useProfileStore();
  const {
    history,
    isLoadingHistory,
    historyError,
    fetchHistory,
    removeFromHistory,
    clearHistory,
  } = useXtreamContentStore();

  const [filterType, setFilterType] = useState<string>('all');

  useEffect(() => {
    if (activeProfile?.id) {
      fetchHistory(activeProfile.id, 50);
    }
  }, [activeProfile?.id, fetchHistory]);

  const filteredHistory = filterType === 'all'
    ? history
    : history.filter(item => item.content_type === filterType);

  const getContentIcon = (contentType: string) => {
    switch (contentType) {
      case 'channel':
        return <RadioIcon />;
      case 'movie':
        return <FilmIcon />;
      case 'series':
        return <TvIcon />;
      default:
        return <PlayIconSmall />;
    }
  };

  const formatDuration = (seconds?: number) => {
    if (!seconds) return '';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    }
    return `${minutes}m`;
  };

  const formatWatchedAt = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 60) {
      return `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;
    } else if (diffHours < 24) {
      return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`;
    } else if (diffDays < 7) {
      return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  const getProgressPercentage = (position?: number, duration?: number) => {
    if (!position || !duration || duration === 0) return 0;
    return Math.min(100, (position / duration) * 100);
  };

  const handleRemove = async (historyId: string) => {
    if (confirm('Remove this item from history?')) {
      try {
        await removeFromHistory(historyId);
      } catch (error) {
        console.error('Failed to remove history item:', error);
      }
    }
  };

  const handleClearAll = async () => {
    if (!activeProfile?.id) return;
    if (confirm('Clear all viewing history? This cannot be undone.')) {
      try {
        await clearHistory(activeProfile.id);
      } catch (error) {
        console.error('Failed to clear history:', error);
      }
    }
  };

  if (!activeProfile) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-gray-400">Please select a profile to view history</p>
      </div>
    );
  }

  if (isLoadingHistory) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500"></div>
      </div>
    );
  }

  if (historyError) {
    return (
      <div className="flex items-center justify-center h-full">
        <p className="text-red-400">Error loading history: {historyError}</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-gray-900 text-white">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-700">
        <div className="flex items-center gap-3">
          <ClockIcon />
          <h2 className="text-xl font-semibold">Viewing History</h2>
        </div>
        <div className="flex items-center gap-3">
          {/* Filter buttons */}
          <div className="flex gap-2">
            <button
              onClick={() => setFilterType('all')}
              className={`px-3 py-1 rounded text-sm ${filterType === 'all'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
            >
              All
            </button>
            <button
              onClick={() => setFilterType('channel')}
              className={`px-3 py-1 rounded text-sm flex items-center gap-1 ${filterType === 'channel'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
            >
              <RadioIcon />
              Channels
            </button>
            <button
              onClick={() => setFilterType('movie')}
              className={`px-3 py-1 rounded text-sm flex items-center gap-1 ${filterType === 'movie'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
            >
              <FilmIcon />
              Movies
            </button>
            <button
              onClick={() => setFilterType('series')}
              className={`px-3 py-1 rounded text-sm flex items-center gap-1 ${filterType === 'series'
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-300 hover:bg-gray-600'
                }`}
            >
              <TvIcon />
              Series
            </button>
          </div>
          {history.length > 0 && (
            <button
              onClick={handleClearAll}
              className="px-3 py-1 rounded text-sm bg-red-600 hover:bg-red-700 text-white flex items-center gap-1"
            >
              <Trash2Icon />
              Clear All
            </button>
          )}
        </div>
      </div>

      {/* History list */}
      <div className="flex-1 overflow-y-auto p-4">
        {filteredHistory.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <div className="w-16 h-16 mb-4 opacity-50">
              <ClockIcon />
            </div>
            <p className="text-lg">No viewing history</p>
            <p className="text-sm mt-2">Content you watch will appear here</p>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredHistory.map((item) => {
              const progress = getProgressPercentage(item.position, item.duration);
              const contentName = item.content_data?.name || item.content_data?.title || 'Unknown';
              const contentImage = item.content_data?.stream_icon || item.content_data?.cover;

              return (
                <div
                  key={item.id}
                  className="bg-gray-800 rounded-lg p-4 hover:bg-gray-750 transition-colors"
                >
                  <div className="flex gap-4">
                    {/* Thumbnail */}
                    <div className="flex-shrink-0 w-32 h-20 bg-gray-700 rounded overflow-hidden">
                      {contentImage ? (
                        <img
                          src={contentImage}
                          alt={contentName}
                          className="w-full h-full object-cover"
                          onError={(e) => {
                            e.currentTarget.style.display = 'none';
                          }}
                        />
                      ) : (
                        <div className="w-full h-full flex items-center justify-center">
                          {getContentIcon(item.content_type)}
                        </div>
                      )}
                    </div>

                    {/* Content info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-2">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            {getContentIcon(item.content_type)}
                            <h3 className="text-lg font-medium truncate">{contentName}</h3>
                          </div>
                          <p className="text-sm text-gray-400 mb-2">
                            Watched {formatWatchedAt(item.watched_at)}
                          </p>
                          {item.position && item.duration && (
                            <div className="space-y-1">
                              <div className="flex items-center gap-2 text-sm text-gray-400">
                                <span>{formatDuration(item.position)}</span>
                                <span>/</span>
                                <span>{formatDuration(item.duration)}</span>
                                <span className="text-blue-400">({Math.round(progress)}%)</span>
                              </div>
                              <div className="w-full bg-gray-700 rounded-full h-1.5">
                                <div
                                  className="bg-blue-500 h-1.5 rounded-full transition-all"
                                  style={{ width: `${progress}%` }}
                                />
                              </div>
                            </div>
                          )}
                        </div>

                        {/* Actions */}
                        <div className="flex gap-2">
                          {onPlayContent && (
                            <button
                              onClick={() => onPlayContent(item)}
                              className="p-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
                              title="Play"
                            >
                              <PlayIconSmall />
                            </button>
                          )}
                          <button
                            onClick={() => handleRemove(item.id)}
                            className="p-2 bg-gray-700 hover:bg-red-600 rounded transition-colors"
                            title="Remove from history"
                          >
                            <Trash2Icon />
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};
