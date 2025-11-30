import React from 'react';
import ProfileManager from '../components/ProfileManager';
import './AuthPage.css';

const AuthPage: React.FC = () => {
  return (
    <div className="auth-page">
      <div className="auth-container">
        <div className="auth-header">
          <h1>Welcome to xTauri</h1>
          <p>Select a profile to continue or create a new one to get started.</p>
        </div>
        <div className="auth-content">
          <ProfileManager />
        </div>
      </div>
    </div>
  );
};

export default AuthPage;
