import React, { useEffect, useState } from 'react';
import { useXtreamContentStore } from '../stores/xtreamContentStore';
import { EnhancedEPGListing } from '../types/types';
import {
  formatEPGTime,
  formatProgramDuration,
  getCurrentProgram,
  getNextProgram,
  getProgramProgress,
  isProgramCurrent,
  isProgramFuture,
  isProgramPast
} from '../utils/epgUtils';

interface EPGDisplayProps {
  profileId: string;
  channelId: string;
  showFullSchedule?: boolean;
  maxPrograms?: number;
  className?: string;
}

interface ProgramItemProps {
  program: EnhancedEPGListing;
  showProgress?: boolean;
}

const ProgramItem: React.FC<ProgramItemProps> = ({ program, showProgress = false }) => {
  const [formattedStart, setFormattedStart] = useState<string>('');
  const [formattedStop, setFormattedStop] = useState<string>('');

  useEffect(() => {
    const formatTimes = async () => {
      if (program.formatted_start) {
        setFormattedStart(program.formatted_start);
      } else {
        const timestamp = program.start_timestamp || parseInt(program.start);
        const formatted = await formatEPGTime(timestamp);
        setFormattedStart(formatted);
      }

      if (program.formatted_stop) {
        setFormattedStop(program.formatted_stop);
      } else {
        const timestamp = program.stop_timestamp || parseInt(program.stop);
        const formatted = await formatEPGTime(timestamp);
        setFormattedStop(formatted);
      }
    };

    formatTimes();
  }, [program]);

  const isCurrent = isProgramCurrent(program);
  const isPast = isProgramPast(program);
  const isFuture = isProgramFuture(program);
  const progress = showProgress && isCurrent ? getProgramProgress(program) : 0;
  const duration = formatProgramDuration(program);

  return (
    <div className={`epg-program ${isCurrent ? 'current' : ''} ${isPast ? 'past' : ''} ${isFuture ? 'future' : ''}`}>
      <div className="epg-program-header">
        <h4 className="epg-program-title">{program.title || 'Unknown Program'}</h4>
        <span className="epg-program-time">
          {formattedStart} - {formattedStop}
        </span>
        <span className="epg-program-duration">({duration})</span>
      </div>

      {program.description && (
        <p className="epg-program-description">{program.description}</p>
      )}

      {showProgress && isCurrent && progress > 0 && (
        <div className="epg-program-progress">
          <div className="epg-progress-bar">
            <div
              className="epg-progress-fill"
              style={{ width: `${progress}%` }}
            />
          </div>
          <span className="epg-progress-text">{progress}% complete</span>
        </div>
      )}

      <div className="epg-program-status">
        {isCurrent && <span className="status-badge current">Now Playing</span>}
        {isPast && <span className="status-badge past">Ended</span>}
        {isFuture && <span className="status-badge future">Upcoming</span>}
      </div>
    </div>
  );
};

const EPGDisplay: React.FC<EPGDisplayProps> = ({
  profileId,
  channelId,
  showFullSchedule = false,
  maxPrograms = 10,
  className = ''
}) => {
  const {
    epgData,
    currentAndNextEPG,
    isLoadingEPG,
    epgError,
    fetchShortEPG,
    fetchFullEPG,
    fetchCurrentAndNextEPG
  } = useXtreamContentStore();

  const [refreshInterval, setRefreshInterval] = useState<number | null>(null);

  useEffect(() => {
    // Initial load
    if (showFullSchedule) {
      fetchFullEPG(profileId, channelId);
    } else {
      fetchCurrentAndNextEPG(profileId, channelId);
      fetchShortEPG(profileId, channelId);
    }

    // Set up auto-refresh for current programs (every 5 minutes)
    const interval = setInterval(() => {
      if (showFullSchedule) {
        fetchFullEPG(profileId, channelId);
      } else {
        fetchCurrentAndNextEPG(profileId, channelId);
      }
    }, 5 * 60 * 1000); // 5 minutes

    setRefreshInterval(interval);

    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [profileId, channelId, showFullSchedule]);

  useEffect(() => {
    return () => {
      if (refreshInterval) {
        clearInterval(refreshInterval);
      }
    };
  }, [refreshInterval]);

  if (isLoadingEPG) {
    return (
      <div className={`epg-display loading ${className}`}>
        <div className="epg-loading">
          <div className="loading-spinner" />
          <span>Loading program guide...</span>
        </div>
      </div>
    );
  }

  if (epgError) {
    return (
      <div className={`epg-display error ${className}`}>
        <div className="epg-error">
          <span>Failed to load program guide: {epgError}</span>
          <button
            onClick={() => showFullSchedule ? fetchFullEPG(profileId, channelId) : fetchCurrentAndNextEPG(profileId, channelId)}
            className="retry-button"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (showFullSchedule) {
    const programs = epgData[channelId] || [];
    const displayPrograms = maxPrograms > 0 ? programs.slice(0, maxPrograms) : programs;

    if (displayPrograms.length === 0) {
      return (
        <div className={`epg-display empty ${className}`}>
          <div className="epg-empty">
            <span>No program guide available for this channel</span>
          </div>
        </div>
      );
    }

    return (
      <div className={`epg-display full-schedule ${className}`}>
        <div className="epg-header">
          <h3>Program Guide</h3>
          <span className="epg-count">{programs.length} programs</span>
        </div>

        <div className="epg-programs">
          {displayPrograms.map((program, index) => (
            <ProgramItem
              key={`${program.id || index}`}
              program={program}
              showProgress={true}
            />
          ))}
        </div>

        {maxPrograms > 0 && programs.length > maxPrograms && (
          <div className="epg-more">
            <span>... and {programs.length - maxPrograms} more programs</span>
          </div>
        )}
      </div>
    );
  } else {
    // Show current and next programs
    const currentNext = currentAndNextEPG[channelId];
    const programs = epgData[channelId] || [];

    // Fallback to manual current/next detection if not available
    const currentProgram = currentNext?.current || getCurrentProgram(programs);
    const nextProgram = currentNext?.next || getNextProgram(programs);

    return (
      <div className={`epg-display current-next ${className}`}>
        <div className="epg-header">
          <h3>Now & Next</h3>
        </div>

        <div className="epg-current-next">
          {currentProgram ? (
            <div className="epg-current">
              <h4>Now Playing</h4>
              <ProgramItem program={currentProgram} showProgress={true} />
            </div>
          ) : (
            <div className="epg-current empty">
              <h4>Now Playing</h4>
              <span>No current program information</span>
            </div>
          )}

          {nextProgram ? (
            <div className="epg-next">
              <h4>Up Next</h4>
              <ProgramItem program={nextProgram} showProgress={false} />
            </div>
          ) : (
            <div className="epg-next empty">
              <h4>Up Next</h4>
              <span>No upcoming program information</span>
            </div>
          )}
        </div>
      </div>
    );
  }
};

export default EPGDisplay;