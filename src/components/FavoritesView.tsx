import React, { useEffect, useState } from 'react';
import { useProfileStore } from '../stores/profileStore';
import { useXtreamContentStore } from '../stores/xtreamContentStore';

// Custom icon components
const HeartFilledIcon = () => (
  <svg className="w-5 h-5" fill="currentColor" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
  </svg>
);

const FilmIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
    <line x1="7" y1="2" x2="7" y2="22" />
    <line x1="17" y1="2" x2="17" y2="22" />
    <line x1="2" y1="12" x2="22" y2="12" />
  </svg>
);

const TvIcon = () => (
  <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
    <rect x="2" y="7" width="20" height="15" rx="2" ry="2" />
    <polyline points="17 2 12 7 7 2" />
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
  </svg>
);

export const FavoritesView: React.FC = () => {
  const { activeProfile } = useProfileStore();
  const {
    favorites,
    isLoadingFavorites,
    favoritesError,
    fetchFavorites,
    removeFromFavorites,
    clearFavorites,
  } = useXtreamContentStore();

  const [filterType, setFilterType] = useState<string>('all');
  const [sortBy, setSortBy] = useState<'date' | 'name'>('date');

  useEffect(() => {
    if (activeProfile?.id) {
      console.log('FavoritesView: Fetching favorites for profile:', activeProfile.id);
      fetchFavorites(activeProfile.id);
    }
  }, [activeProfile?.id, fetchFavorites]);

  const filteredFavorites = filterType === 'all'
    ? favorites
    : favorites.filter(item => item.content_type === filterType);

  const sortedFavorites = [...filteredFavorites].sort((a, b) => {
    if (sortBy === 'date') {
      return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
    } else {
      const nameA = a.content_data?.name || a.content_data?.title || '';
      const nameB = b.content_data?.name || b.content_data?.title || '';
      return nameA.localeCompare(nameB);
    }
  });

  const getContentIcon = (contentType: string) => {
    switch (contentType) {
      case 'channel':
        return <RadioIcon />;
      case 'movie':
        return <FilmIcon />;
      case 'series':
        return <TvIcon />;
      default:
        return <RadioIcon />;
    }
  };

  const formatAddedAt = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffDays === 0) return 'Added today';
    if (diffDays === 1) return 'Added yesterday';
    if (diffDays < 7) return `Added ${diffDays} days ago`;
    return `Added ${date.toLocaleDateString()}`;
  };

  const handleRemove = async (favoriteId: string) => {
    if (confirm('Remove this item from favorites?')) {
      try {
        await removeFromFavorites(favoriteId);
      } catch (error) {
        console.error('Failed to remove favorite:', error);
      }
    }
  };

  const handleClearAll = async () => {
    if (!activeProfile?.id) return;
    if (confirm('Clear all favorites? This cannot be undone.')) {
      try {
        await clearFavorites(activeProfile.id);
      } catch (error) {
        console.error('Failed to clear favorites:', error);
      }
    }
  };

  if (!activeProfile) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <p style={{ color: '#9CA3AF' }}>Please select a profile to view favorites</p>
      </div>
    );
  }

  if (isLoadingFavorites) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <div style={{ width: '48px', height: '48px', border: '2px solid #DB2777', borderTopColor: 'transparent', borderRadius: '50%', animation: 'spin 1s linear infinite' }}></div>
      </div>
    );
  }

  if (favoritesError) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>
        <p style={{ color: '#F87171' }}>Error loading favorites: {favoritesError}</p>
      </div>
    );
  }

  return (
    <div className="favorites-view" style={{ padding: '20px', height: '100%', display: 'flex', flexDirection: 'column' }}>
      {/* Header */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '20px', paddingBottom: '15px', borderBottom: '1px solid #374151' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <HeartFilledIcon />
          <h2 style={{ fontSize: '1.5rem', fontWeight: '600', margin: 0 }}>Favorites</h2>
          <span style={{ fontSize: '0.875rem', color: '#9CA3AF' }}>({sortedFavorites.length})</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', flexWrap: 'wrap' }}>
          {/* Sort dropdown */}
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as 'date' | 'name')}
            style={{ padding: '6px 12px', borderRadius: '4px', fontSize: '0.875rem', backgroundColor: '#374151', color: '#D1D5DB', border: '1px solid #4B5563' }}
          >
            <option value="date">Sort by Date</option>
            <option value="name">Sort by Name</option>
          </select>

          {/* Filter buttons */}
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={() => setFilterType('all')}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                fontSize: '0.875rem',
                backgroundColor: filterType === 'all' ? '#DB2777' : '#374151',
                color: 'white',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              All
            </button>
            <button
              onClick={() => setFilterType('channel')}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                fontSize: '0.875rem',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                backgroundColor: filterType === 'channel' ? '#DB2777' : '#374151',
                color: 'white',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              <RadioIcon />
              Channels
            </button>
            <button
              onClick={() => setFilterType('movie')}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                fontSize: '0.875rem',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                backgroundColor: filterType === 'movie' ? '#DB2777' : '#374151',
                color: 'white',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              <FilmIcon />
              Movies
            </button>
            <button
              onClick={() => setFilterType('series')}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                fontSize: '0.875rem',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                backgroundColor: filterType === 'series' ? '#DB2777' : '#374151',
                color: 'white',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              <TvIcon />
              Series
            </button>
          </div>
          {favorites.length > 0 && (
            <button
              onClick={handleClearAll}
              style={{
                padding: '6px 12px',
                borderRadius: '4px',
                fontSize: '0.875rem',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
                backgroundColor: '#DC2626',
                color: 'white',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              <Trash2Icon />
              Clear All
            </button>
          )}
        </div>
      </div>

      {/* Favorites grid */}
      <div style={{ flex: 1, overflowY: 'auto' }}>
        {sortedFavorites.length === 0 ? (
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '400px', color: '#9CA3AF' }}>
            <div style={{ width: '64px', height: '64px', marginBottom: '16px', opacity: 0.5 }}>
              <HeartFilledIcon />
            </div>
            <p style={{ fontSize: '1.125rem' }}>No favorites yet</p>
            <p style={{ fontSize: '0.875rem', marginTop: '8px' }}>Add content to your favorites to see it here</p>
          </div>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))', gap: '16px' }}>
            {sortedFavorites.map((item) => {
              const contentName = item.content_data?.name || item.content_data?.title || 'Unknown';
              const contentImage = item.content_data?.stream_icon || item.content_data?.cover;

              return (
                <div
                  key={item.id}
                  style={{
                    backgroundColor: '#1F2937',
                    borderRadius: '8px',
                    overflow: 'hidden',
                    transition: 'all 0.2s',
                    cursor: 'pointer'
                  }}
                  onMouseEnter={(e) => e.currentTarget.style.boxShadow = '0 0 0 2px #DB2777'}
                  onMouseLeave={(e) => e.currentTarget.style.boxShadow = 'none'}
                >
                  {/* Thumbnail */}
                  <div style={{ position: 'relative', width: '100%', paddingBottom: '56.25%', backgroundColor: '#374151' }}>
                    {contentImage ? (
                      <img
                        src={contentImage}
                        alt={contentName}
                        style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', objectFit: 'cover' }}
                        onError={(e) => {
                          e.currentTarget.style.display = 'none';
                        }}
                      />
                    ) : (
                      <div style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#6B7280' }}>
                        {getContentIcon(item.content_type)}
                      </div>
                    )}

                    {/* Content type badge */}
                    <div style={{ position: 'absolute', top: '8px', left: '8px', padding: '4px 8px', backgroundColor: 'rgba(0,0,0,0.75)', borderRadius: '4px', fontSize: '0.75rem', display: 'flex', alignItems: 'center', gap: '4px' }}>
                      {getContentIcon(item.content_type)}
                      <span style={{ textTransform: 'capitalize' }}>{item.content_type}</span>
                    </div>

                    {/* Remove button */}
                    <div style={{ position: 'absolute', bottom: '8px', right: '8px' }}>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRemove(item.id);
                        }}
                        style={{
                          padding: '8px',
                          backgroundColor: '#DC2626',
                          borderRadius: '50%',
                          border: 'none',
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center'
                        }}
                        title="Remove from favorites"
                      >
                        <Trash2Icon />
                      </button>
                    </div>
                  </div>

                  {/* Content info */}
                  <div style={{ padding: '12px' }}>
                    <h3 style={{ fontSize: '0.875rem', fontWeight: '500', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', marginBottom: '4px' }} title={contentName}>
                      {contentName}
                    </h3>
                    <p style={{ fontSize: '0.75rem', color: '#9CA3AF' }}>
                      {formatAddedAt(item.created_at)}
                    </p>
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
