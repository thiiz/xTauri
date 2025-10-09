import { useEffect, useState } from 'react';
import { useProfileStore, useXtreamContentStore } from '../stores';
import type { SyncSettings } from '../stores/xtreamContentStore';

export default function ContentSyncManager() {
  const { activeProfile } = useProfileStore();
  const {
    syncProgress,
    syncSettings,
    cacheStats,
    isSyncing,
    syncError,
    startContentSync,
    cancelContentSync,
    getSyncSettings,
    updateSyncSettings,
    clearContentCache,
    getContentCacheStats,
  } = useXtreamContentStore();

  const [showSettings, setShowSettings] = useState(false);
  const [localSettings, setLocalSettings] = useState<SyncSettings | null>(null);

  useEffect(() => {
    if (activeProfile) {
      getSyncSettings(activeProfile.id).catch(console.error);
      getContentCacheStats(activeProfile.id).catch(console.error);
    }
  }, [activeProfile, getSyncSettings, getContentCacheStats]);

  useEffect(() => {
    if (syncSettings) {
      setLocalSettings(syncSettings);
    }
  }, [syncSettings]);

  const handleStartSync = async (fullSync: boolean) => {
    if (!activeProfile) return;
    try {
      await startContentSync(activeProfile.id, fullSync);
    } catch (error) {
      console.error('Failed to start sync:', error);
    }
  };

  const handleCancelSync = async () => {
    if (!activeProfile) return;
    try {
      await cancelContentSync(activeProfile.id);
    } catch (error) {
      console.error('Failed to cancel sync:', error);
    }
  };

  const handleSaveSettings = async () => {
    if (!activeProfile || !localSettings) return;
    try {
      await updateSyncSettings(activeProfile.id, localSettings);
      setShowSettings(false);
    } catch (error) {
      console.error('Failed to update sync settings:', error);
    }
  };

  const handleClearCache = async () => {
    if (!activeProfile) return;
    if (!confirm('Are you sure you want to clear the content cache? This will remove all cached channels, movies, and series.')) {
      return;
    }
    try {
      await clearContentCache(activeProfile.id);
    } catch (error) {
      console.error('Failed to clear cache:', error);
    }
  };

  if (!activeProfile) {
    return null;
  }

  const getStatusColor = (status?: string) => {
    switch (status) {
      case 'syncing': return '#3b82f6';
      case 'completed': return '#10b981';
      case 'failed': return '#ef4444';
      case 'cancelled': return '#f59e0b';
      default: return '#6b7280';
    }
  };

  return (
    <div className="sync-manager">
      <div className="sync-header">
        <h3>Content Synchronization</h3>
        <button
          className="settings-btn"
          onClick={() => setShowSettings(!showSettings)}
          title="Sync Settings"
        >
          ⚙️
        </button>
      </div>

      {syncError && (
        <div className="sync-error">
          <span>⚠️ {syncError}</span>
        </div>
      )}

      {cacheStats && (
        <div className="cache-stats">
          <div className="stat-item">
            <span className="stat-label">Channels:</span>
            <span className="stat-value">{cacheStats.channels_count}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Movies:</span>
            <span className="stat-value">{cacheStats.movies_count}</span>
          </div>
          <div className="stat-item">
            <span className="stat-label">Series:</span>
            <span className="stat-value">{cacheStats.series_count}</span>
          </div>
        </div>
      )}

      {syncProgress && (
        <div className="sync-progress">
          <div className="progress-header">
            <span style={{ color: getStatusColor(syncProgress.status) }}>
              {syncProgress.status.toUpperCase()}
            </span>
            <span>{syncProgress.progress}%</span>
          </div>
          <div className="progress-bar">
            <div
              className="progress-fill"
              style={{
                width: `${syncProgress.progress}%`,
                backgroundColor: getStatusColor(syncProgress.status)
              }}
            />
          </div>
          <div className="progress-details">
            <span>{syncProgress.current_step}</span>
            <div className="progress-counts">
              <span>📺 {syncProgress.channels_synced}</span>
              <span>🎬 {syncProgress.movies_synced}</span>
              <span>📺 {syncProgress.series_synced}</span>
            </div>
          </div>
        </div>
      )}

      <div className="sync-actions">
        {isSyncing ? (
          <button
            className="btn btn-danger"
            onClick={handleCancelSync}
          >
            Cancel Sync
          </button>
        ) : (
          <>
            <button
              className="btn btn-primary"
              onClick={() => handleStartSync(false)}
              title="Sync only new and updated content"
            >
              Incremental Sync
            </button>
            <button
              className="btn btn-secondary"
              onClick={() => handleStartSync(true)}
              title="Sync all content from scratch"
            >
              Full Sync
            </button>
            <button
              className="btn btn-danger"
              onClick={handleClearCache}
              title="Clear all cached content"
            >
              Clear Cache
            </button>
          </>
        )}
      </div>

      {showSettings && localSettings && (
        <div className="sync-settings">
          <h4>Sync Settings</h4>
          <div className="setting-item">
            <label>
              <input
                type="checkbox"
                checked={localSettings.auto_sync_enabled}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  auto_sync_enabled: e.target.checked
                })}
              />
              Enable Auto Sync
            </label>
          </div>
          <div className="setting-item">
            <label>
              <input
                type="checkbox"
                checked={localSettings.sync_on_startup}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  sync_on_startup: e.target.checked
                })}
              />
              Sync on Startup
            </label>
          </div>
          <div className="setting-item">
            <label>
              Sync Interval (hours):
              <input
                type="number"
                min="1"
                max="168"
                value={localSettings.sync_interval_hours}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  sync_interval_hours: parseInt(e.target.value) || 24
                })}
              />
            </label>
          </div>
          <div className="setting-item">
            <label>
              <input
                type="checkbox"
                checked={localSettings.sync_channels}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  sync_channels: e.target.checked
                })}
              />
              Sync Channels
            </label>
          </div>
          <div className="setting-item">
            <label>
              <input
                type="checkbox"
                checked={localSettings.sync_movies}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  sync_movies: e.target.checked
                })}
              />
              Sync Movies
            </label>
          </div>
          <div className="setting-item">
            <label>
              <input
                type="checkbox"
                checked={localSettings.sync_series}
                onChange={(e) => setLocalSettings({
                  ...localSettings,
                  sync_series: e.target.checked
                })}
              />
              Sync Series
            </label>
          </div>
          <div className="settings-actions">
            <button className="btn btn-primary" onClick={handleSaveSettings}>
              Save Settings
            </button>
            <button className="btn btn-secondary" onClick={() => setShowSettings(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}

      <style>{`
        .sync-manager {
          padding: 1rem;
          background: var(--bg-secondary, #1e1e1e);
          border-radius: 8px;
          margin-bottom: 1rem;
        }

        .sync-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1rem;
        }

        .sync-header h3 {
          margin: 0;
          font-size: 1.1rem;
        }

        .settings-btn {
          background: none;
          border: none;
          font-size: 1.2rem;
          cursor: pointer;
          padding: 0.25rem;
        }

        .sync-error {
          padding: 0.75rem;
          background: #ef444420;
          border: 1px solid #ef4444;
          border-radius: 4px;
          margin-bottom: 1rem;
          color: #ef4444;
        }

        .cache-stats {
          display: flex;
          gap: 1rem;
          margin-bottom: 1rem;
          padding: 0.75rem;
          background: var(--bg-tertiary, #2a2a2a);
          border-radius: 4px;
        }

        .stat-item {
          display: flex;
          flex-direction: column;
          gap: 0.25rem;
        }

        .stat-label {
          font-size: 0.85rem;
          opacity: 0.7;
        }

        .stat-value {
          font-size: 1.2rem;
          font-weight: bold;
        }

        .sync-progress {
          margin-bottom: 1rem;
        }

        .progress-header {
          display: flex;
          justify-content: space-between;
          margin-bottom: 0.5rem;
          font-weight: bold;
        }

        .progress-bar {
          height: 8px;
          background: var(--bg-tertiary, #2a2a2a);
          border-radius: 4px;
          overflow: hidden;
          margin-bottom: 0.5rem;
        }

        .progress-fill {
          height: 100%;
          transition: width 0.3s ease;
        }

        .progress-details {
          display: flex;
          justify-content: space-between;
          font-size: 0.85rem;
          opacity: 0.8;
        }

        .progress-counts {
          display: flex;
          gap: 1rem;
        }

        .sync-actions {
          display: flex;
          gap: 0.5rem;
          flex-wrap: wrap;
        }

        .btn {
          padding: 0.5rem 1rem;
          border: none;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.9rem;
          transition: opacity 0.2s;
        }

        .btn:hover {
          opacity: 0.8;
        }

        .btn-primary {
          background: #3b82f6;
          color: white;
        }

        .btn-secondary {
          background: #6b7280;
          color: white;
        }

        .btn-danger {
          background: #ef4444;
          color: white;
        }

        .sync-settings {
          margin-top: 1rem;
          padding: 1rem;
          background: var(--bg-tertiary, #2a2a2a);
          border-radius: 4px;
        }

        .sync-settings h4 {
          margin: 0 0 1rem 0;
        }

        .setting-item {
          margin-bottom: 0.75rem;
        }

        .setting-item label {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          cursor: pointer;
        }

        .setting-item input[type="checkbox"] {
          cursor: pointer;
        }

        .setting-item input[type="number"] {
          width: 80px;
          padding: 0.25rem;
          margin-left: 0.5rem;
          background: var(--bg-secondary, #1e1e1e);
          border: 1px solid #444;
          border-radius: 4px;
          color: inherit;
        }

        .settings-actions {
          display: flex;
          gap: 0.5rem;
          margin-top: 1rem;
        }
      `}</style>
    </div>
  );
}
