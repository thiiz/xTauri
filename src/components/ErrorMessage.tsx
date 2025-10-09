import React from 'react';
import '../styles/ErrorMessage.css';

export type ErrorSeverity = 'error' | 'warning' | 'info';

interface ErrorMessageProps {
  title?: string;
  message: string;
  severity?: ErrorSeverity;
  details?: string;
  onRetry?: () => void;
  onDismiss?: () => void;
  retryLabel?: string;
  dismissLabel?: string;
  showIcon?: boolean;
}

export const ErrorMessage: React.FC<ErrorMessageProps> = ({
  title,
  message,
  severity = 'error',
  details,
  onRetry,
  onDismiss,
  retryLabel = 'Retry',
  dismissLabel = 'Dismiss',
  showIcon = true,
}) => {
  const getIcon = () => {
    switch (severity) {
      case 'error':
        return (
          <svg className="error-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10" strokeWidth="2" />
            <line x1="12" y1="8" x2="12" y2="12" strokeWidth="2" strokeLinecap="round" />
            <line x1="12" y1="16" x2="12.01" y2="16" strokeWidth="2" strokeLinecap="round" />
          </svg>
        );
      case 'warning':
        return (
          <svg className="error-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" strokeWidth="2" />
            <line x1="12" y1="9" x2="12" y2="13" strokeWidth="2" strokeLinecap="round" />
            <line x1="12" y1="17" x2="12.01" y2="17" strokeWidth="2" strokeLinecap="round" />
          </svg>
        );
      case 'info':
        return (
          <svg className="error-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
            <circle cx="12" cy="12" r="10" strokeWidth="2" />
            <line x1="12" y1="16" x2="12" y2="12" strokeWidth="2" strokeLinecap="round" />
            <line x1="12" y1="8" x2="12.01" y2="8" strokeWidth="2" strokeLinecap="round" />
          </svg>
        );
    }
  };

  return (
    <div className={`error-message error-message-${severity}`}>
      {showIcon && <div className="error-icon-container">{getIcon()}</div>}
      <div className="error-content">
        {title && <h3 className="error-title">{title}</h3>}
        <p className="error-text">{message}</p>
        {details && (
          <details className="error-details">
            <summary>Technical Details</summary>
            <pre className="error-details-content">{details}</pre>
          </details>
        )}
        {(onRetry || onDismiss) && (
          <div className="error-actions">
            {onRetry && (
              <button className="error-button error-button-primary" onClick={onRetry}>
                {retryLabel}
              </button>
            )}
            {onDismiss && (
              <button className="error-button error-button-secondary" onClick={onDismiss}>
                {dismissLabel}
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

interface ErrorBoundaryProps {
  children: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}

export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="error-boundary-fallback">
          <ErrorMessage
            title="Something went wrong"
            message={this.state.error?.message || 'An unexpected error occurred'}
            severity="error"
            details={this.state.error?.stack}
            onRetry={this.handleReset}
            retryLabel="Try Again"
          />
        </div>
      );
    }

    return this.props.children;
  }
}

// Helper function to get user-friendly error messages
export const getUserFriendlyErrorMessage = (error: unknown): string => {
  if (typeof error === 'string') {
    return error;
  }

  if (error instanceof Error) {
    // Network errors
    if (error.message.includes('Network') || error.message.includes('fetch')) {
      return 'Unable to connect to the server. Please check your internet connection and try again.';
    }

    // Authentication errors
    if (error.message.includes('auth') || error.message.includes('credentials')) {
      return 'Authentication failed. Please check your username and password.';
    }

    // Timeout errors
    if (error.message.includes('timeout')) {
      return 'The request took too long to complete. Please try again.';
    }

    // Server errors
    if (error.message.includes('500') || error.message.includes('server error')) {
      return 'The server encountered an error. Please try again later.';
    }

    // Not found errors
    if (error.message.includes('404') || error.message.includes('not found')) {
      return 'The requested content could not be found.';
    }

    return error.message;
  }

  return 'An unexpected error occurred. Please try again.';
};

// Helper function to get error recovery suggestions
export const getErrorRecoverySuggestions = (error: unknown): string[] => {
  const suggestions: string[] = [];

  if (typeof error === 'string' || error instanceof Error) {
    const errorMessage = typeof error === 'string' ? error : error.message;

    if (errorMessage.includes('Network') || errorMessage.includes('fetch')) {
      suggestions.push('Check your internet connection');
      suggestions.push('Verify the server URL is correct');
      suggestions.push('Try again in a few moments');
    }

    if (errorMessage.includes('auth') || errorMessage.includes('credentials')) {
      suggestions.push('Verify your username and password');
      suggestions.push('Check if your account is active');
      suggestions.push('Contact your service provider if the issue persists');
    }

    if (errorMessage.includes('timeout')) {
      suggestions.push('Check your internet speed');
      suggestions.push('Try again with a more stable connection');
      suggestions.push('Contact support if timeouts persist');
    }

    if (errorMessage.includes('500') || errorMessage.includes('server error')) {
      suggestions.push('Wait a few minutes and try again');
      suggestions.push('Check if the service is experiencing issues');
      suggestions.push('Contact your service provider');
    }
  }

  if (suggestions.length === 0) {
    suggestions.push('Try refreshing the page');
    suggestions.push('Clear your cache and try again');
    suggestions.push('Contact support if the problem continues');
  }

  return suggestions;
};
