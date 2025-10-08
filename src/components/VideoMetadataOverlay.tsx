import type { EnhancedEPGListing } from "../types/types";

interface VideoMetadataOverlayProps {
  show: boolean;
  title: string;
  statusText: string;
  qualityBadge: string;
  currentEPG: EnhancedEPGListing | null;
  metadata: {
    genre?: string | null;
    year?: string | null;
    rating?: number;
    duration?: number | null;
    cast?: string | null;
    director?: string | null;
    description?: string | null;
  } | null;
  onToggle: () => void;
}

const VideoMetadataOverlay: React.FC<VideoMetadataOverlayProps> = ({
  show,
  title,
  statusText,
  qualityBadge,
  currentEPG,
  metadata,
  onToggle,
}) => {
  return (
    <div className={`video-metadata-overlay ${show ? 'show' : ''}`}>
      <button
        className="metadata-toggle-btn"
        onClick={onToggle}
        title="Toggle metadata (I)"
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z" />
        </svg>
      </button>

      <div className="metadata-content">
        <div className="metadata-header">
          <h3 className="metadata-title">{title}</h3>
          <div className="metadata-badges">
            <span className="status-badge">{statusText}</span>
            <span className="quality-badge">{qualityBadge}</span>
          </div>
        </div>

        {/* EPG Information for Live Channels */}
        {currentEPG && (
          <div className="epg-section">
            <div className="epg-program">
              <div className="program-title">{currentEPG.title}</div>
              <div className="program-time">
                {new Date((currentEPG.start_timestamp || 0) * 1000).toLocaleTimeString([], {
                  hour: '2-digit',
                  minute: '2-digit'
                })} - {new Date((currentEPG.stop_timestamp || 0) * 1000).toLocaleTimeString([], {
                  hour: '2-digit',
                  minute: '2-digit'
                })}
              </div>
              {currentEPG.description && (
                <div className="program-description">{currentEPG.description}</div>
              )}
            </div>
          </div>
        )}

        {/* Metadata for Movies and Series */}
        {metadata && (
          <div className="content-metadata-section">
            <div className="metadata-tags">
              {metadata.year && (
                <span className="metadata-tag year-tag">{metadata.year}</span>
              )}
              {metadata.genre && (
                <span className="metadata-tag genre-tag">{metadata.genre}</span>
              )}
              {metadata.rating && metadata.rating > 0 && (
                <span className="metadata-tag rating-tag">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
                    <path d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z" />
                  </svg>
                  {metadata.rating.toFixed(1)}
                </span>
              )}
              {metadata.duration && (
                <span className="metadata-tag duration-tag">
                  {Math.floor(metadata.duration / 60)}h {metadata.duration % 60}m
                </span>
              )}
            </div>

            {metadata.description && (
              <div className="metadata-description">{metadata.description}</div>
            )}

            {(metadata.director || metadata.cast) && (
              <div className="metadata-credits">
                {metadata.director && (
                  <div className="credit-item">
                    <span className="credit-label">Director:</span>
                    <span className="credit-value">{metadata.director}</span>
                  </div>
                )}
                {metadata.cast && (
                  <div className="credit-item">
                    <span className="credit-label">Cast:</span>
                    <span className="credit-value">{metadata.cast}</span>
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default VideoMetadataOverlay;
