import React from 'react';
import '../styles/LoadingIndicator.css';

interface LoadingIndicatorProps {
  message?: string;
  progress?: number;
  size?: 'small' | 'medium' | 'large';
  variant?: 'spinner' | 'bar' | 'dots';
}

export const LoadingIndicator: React.FC<LoadingIndicatorProps> = ({
  message = 'Loading...',
  progress,
  size = 'medium',
  variant = 'spinner',
}) => {
  const renderSpinner = () => (
    <div className={`loading-spinner loading-spinner-${size}`}>
      <div className="spinner-circle"></div>
    </div>
  );

  const renderProgressBar = () => (
    <div className="loading-progress-bar">
      <div
        className="progress-bar-fill"
        style={{ width: `${progress || 0}%` }}
      ></div>
    </div>
  );

  const renderDots = () => (
    <div className="loading-dots">
      <span className="dot"></span>
      <span className="dot"></span>
      <span className="dot"></span>
    </div>
  );

  return (
    <div className={`loading-indicator loading-indicator-${size}`}>
      {variant === 'spinner' && renderSpinner()}
      {variant === 'bar' && renderProgressBar()}
      {variant === 'dots' && renderDots()}
      {message && <p className="loading-message">{message}</p>}
      {progress !== undefined && variant === 'bar' && (
        <p className="loading-progress">{Math.round(progress)}%</p>
      )}
    </div>
  );
};

interface LoadingOverlayProps {
  message?: string;
  progress?: number;
  transparent?: boolean;
}

export const LoadingOverlay: React.FC<LoadingOverlayProps> = ({
  message,
  progress,
  transparent = false,
}) => {
  return (
    <div className={`loading-overlay ${transparent ? 'transparent' : ''}`}>
      <div className="loading-overlay-content">
        <LoadingIndicator
          message={message}
          progress={progress}
          size="large"
          variant={progress !== undefined ? 'bar' : 'spinner'}
        />
      </div>
    </div>
  );
};

interface SkeletonLoaderProps {
  count?: number;
  height?: string;
  width?: string;
  variant?: 'text' | 'rectangular' | 'circular';
}

export const SkeletonLoader: React.FC<SkeletonLoaderProps> = ({
  count = 1,
  height = '20px',
  width = '100%',
  variant = 'text',
}) => {
  return (
    <>
      {Array.from({ length: count }).map((_, index) => (
        <div
          key={index}
          className={`skeleton skeleton-${variant}`}
          style={{ height, width }}
        ></div>
      ))}
    </>
  );
};
