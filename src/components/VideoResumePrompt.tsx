interface VideoResumePromptProps {
  resumePosition: number;
  onResume: () => void;
  onStartOver: () => void;
}

const VideoResumePrompt: React.FC<VideoResumePromptProps> = ({
  resumePosition,
  onResume,
  onStartOver,
}) => {
  const formatTime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = Math.floor(seconds % 60);

    if (hours > 0) {
      return `${hours}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="video-resume-prompt">
      <div className="resume-prompt-content">
        <div className="resume-icon">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 14.5v-9l6 4.5-6 4.5z" />
          </svg>
        </div>

        <h3 className="resume-title">Resume Playback</h3>
        <p className="resume-message">
          Continue watching from <strong>{formatTime(resumePosition)}</strong>?
        </p>

        <div className="resume-actions">
          <button
            className="resume-btn primary-btn"
            onClick={onResume}
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M8 5v14l11-7z" />
            </svg>
            Resume
          </button>
          <button
            className="resume-btn secondary-btn"
            onClick={onStartOver}
          >
            <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
              <path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z" />
            </svg>
            Start Over
          </button>
        </div>
      </div>
    </div>
  );
};

export default VideoResumePrompt;
