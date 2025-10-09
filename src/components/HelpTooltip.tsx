import React, { useState } from 'react';
import '../styles/HelpTooltip.css';

interface HelpTooltipProps {
  content: string | React.ReactNode;
  title?: string;
  position?: 'top' | 'bottom' | 'left' | 'right';
  maxWidth?: string;
}

export const HelpTooltip: React.FC<HelpTooltipProps> = ({
  content,
  title,
  position = 'top',
  maxWidth = '300px',
}) => {
  const [isVisible, setIsVisible] = useState(false);

  return (
    <div className="help-tooltip-container">
      <button
        className="help-tooltip-trigger"
        onMouseEnter={() => setIsVisible(true)}
        onMouseLeave={() => setIsVisible(false)}
        onClick={() => setIsVisible(!isVisible)}
        aria-label="Help"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <circle cx="12" cy="12" r="10" strokeWidth="2" />
          <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          <line x1="12" y1="17" x2="12.01" y2="17" strokeWidth="2" strokeLinecap="round" />
        </svg>
      </button>
      {isVisible && (
        <div
          className={`help-tooltip-content help-tooltip-${position}`}
          style={{ maxWidth }}
        >
          {title && <div className="help-tooltip-title">{title}</div>}
          <div className="help-tooltip-body">{content}</div>
        </div>
      )}
    </div>
  );
};

interface OnboardingStep {
  title: string;
  content: string | React.ReactNode;
  target?: string;
}

interface OnboardingTourProps {
  steps: OnboardingStep[];
  onComplete?: () => void;
  onSkip?: () => void;
}

export const OnboardingTour: React.FC<OnboardingTourProps> = ({
  steps,
  onComplete,
  onSkip,
}) => {
  const [currentStep, setCurrentStep] = useState(0);
  const [isVisible, setIsVisible] = useState(true);

  const handleNext = () => {
    if (currentStep < steps.length - 1) {
      setCurrentStep(currentStep + 1);
    } else {
      handleComplete();
    }
  };

  const handlePrevious = () => {
    if (currentStep > 0) {
      setCurrentStep(currentStep - 1);
    }
  };

  const handleComplete = () => {
    setIsVisible(false);
    if (onComplete) {
      onComplete();
    }
  };

  const handleSkip = () => {
    setIsVisible(false);
    if (onSkip) {
      onSkip();
    }
  };

  if (!isVisible || steps.length === 0) {
    return null;
  }

  const step = steps[currentStep];

  return (
    <div className="onboarding-overlay">
      <div className="onboarding-content">
        <div className="onboarding-header">
          <h2 className="onboarding-title">{step.title}</h2>
          <button className="onboarding-close" onClick={handleSkip} aria-label="Close">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <line x1="18" y1="6" x2="6" y2="18" strokeWidth="2" strokeLinecap="round" />
              <line x1="6" y1="6" x2="18" y2="18" strokeWidth="2" strokeLinecap="round" />
            </svg>
          </button>
        </div>
        <div className="onboarding-body">{step.content}</div>
        <div className="onboarding-footer">
          <div className="onboarding-progress">
            {steps.map((_, index) => (
              <div
                key={index}
                className={`progress-dot ${index === currentStep ? 'active' : ''} ${index < currentStep ? 'completed' : ''
                  }`}
              />
            ))}
          </div>
          <div className="onboarding-actions">
            {currentStep > 0 && (
              <button className="onboarding-button secondary" onClick={handlePrevious}>
                Previous
              </button>
            )}
            <button className="onboarding-button secondary" onClick={handleSkip}>
              Skip Tour
            </button>
            <button className="onboarding-button primary" onClick={handleNext}>
              {currentStep < steps.length - 1 ? 'Next' : 'Get Started'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

interface HelpPanelProps {
  title: string;
  sections: Array<{
    title: string;
    content: string | React.ReactNode;
  }>;
  onClose?: () => void;
}

export const HelpPanel: React.FC<HelpPanelProps> = ({ title, sections, onClose }) => {
  const [expandedSection, setExpandedSection] = useState<number | null>(0);

  const toggleSection = (index: number) => {
    setExpandedSection(expandedSection === index ? null : index);
  };

  return (
    <div className="help-panel">
      <div className="help-panel-header">
        <h2 className="help-panel-title">{title}</h2>
        {onClose && (
          <button className="help-panel-close" onClick={onClose} aria-label="Close">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor">
              <line x1="18" y1="6" x2="6" y2="18" strokeWidth="2" strokeLinecap="round" />
              <line x1="6" y1="6" x2="18" y2="18" strokeWidth="2" strokeLinecap="round" />
            </svg>
          </button>
        )}
      </div>
      <div className="help-panel-content">
        {sections.map((section, index) => (
          <div key={index} className="help-section">
            <button
              className="help-section-header"
              onClick={() => toggleSection(index)}
              aria-expanded={expandedSection === index}
            >
              <span className="help-section-title">{section.title}</span>
              <svg
                className={`help-section-icon ${expandedSection === index ? 'expanded' : ''}`}
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
              >
                <polyline points="6 9 12 15 18 9" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </button>
            {expandedSection === index && (
              <div className="help-section-content">{section.content}</div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};
