import "../styles/skeleton-loader.css";

interface SkeletonCardProps {
  count?: number;
}

export function SkeletonMovieCard() {
  return (
    <div className="skeleton-card">
      <div className="skeleton-poster"></div>
      <div className="skeleton-info">
        <div className="skeleton-title"></div>
        <div className="skeleton-meta">
          <div className="skeleton-badge"></div>
          <div className="skeleton-badge"></div>
        </div>
      </div>
    </div>
  );
}

export function SkeletonMovieGrid({ count = 18 }: SkeletonCardProps) {
  return (
    <div className="skeleton-grid">
      {Array.from({ length: count }).map((_, index) => (
        <SkeletonMovieCard key={index} />
      ))}
    </div>
  );
}

export function SkeletonEpisodeCard() {
  return (
    <div className="skeleton-episode-card">
      <div className="skeleton-episode-thumbnail"></div>
      <div className="skeleton-episode-content">
        <div className="skeleton-episode-title"></div>
        <div className="skeleton-episode-meta">
          <div className="skeleton-badge-small"></div>
          <div className="skeleton-badge-small"></div>
        </div>
        <div className="skeleton-episode-plot"></div>
      </div>
    </div>
  );
}

export function SkeletonEpisodeList({ count = 10 }: SkeletonCardProps) {
  return (
    <div className="skeleton-episode-list">
      {Array.from({ length: count }).map((_, index) => (
        <SkeletonEpisodeCard key={index} />
      ))}
    </div>
  );
}

export function LoadingSpinner({ text = "Loading...", subtext }: { text?: string; subtext?: string }) {
  return (
    <div className="loading-state-container">
      <div className="loading-spinner" role="status" aria-label="Loading"></div>
      <div className="loading-text">{text}</div>
      {subtext && <div className="loading-subtext">{subtext}</div>}
    </div>
  );
}
