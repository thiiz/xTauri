import React, { useState } from 'react';
import { useXtreamContentStore } from '../stores/xtreamContentStore';
import {
  EPGTimeFilters,
  formatEPGTime,
  getCurrentTimestamp,
  getTimestampHoursFromNow
} from '../utils/epgUtils';
import EPGDisplay from './EPGDisplay';

const EPGTest: React.FC = () => {
  const [profileId, setProfileId] = useState('test-profile');
  const [channelId, setChannelId] = useState('');
  const [showFullSchedule, setShowFullSchedule] = useState(false);
  const [testResults, setTestResults] = useState<string[]>([]);

  const { clearEPG } = useXtreamContentStore();

  const addTestResult = (result: string) => {
    setTestResults(prev => [...prev, `${new Date().toLocaleTimeString()}: ${result}`]);
  };

  const testEPGUtilities = async () => {
    try {
      addTestResult('Testing EPG utilities...');

      // Test timestamp functions
      const currentTimestamp = await getCurrentTimestamp();
      addTestResult(`Current timestamp: ${currentTimestamp}`);

      const futureTimestamp = await getTimestampHoursFromNow(6);
      addTestResult(`6 hours from now: ${futureTimestamp}`);

      // Test time formatting
      const formattedTime = await formatEPGTime(currentTimestamp);
      addTestResult(`Formatted current time: ${formattedTime}`);

      // Test time filters
      const todayFilter = await EPGTimeFilters.today();
      addTestResult(`Today filter: ${todayFilter.start_timestamp} - ${todayFilter.end_timestamp}`);

      const next24HoursFilter = await EPGTimeFilters.next24Hours();
      addTestResult(`Next 24 hours filter: ${next24HoursFilter.start_timestamp} - ${next24HoursFilter.end_timestamp}`);

      addTestResult('EPG utilities test completed successfully!');
    } catch (error) {
      addTestResult(`EPG utilities test failed: ${error}`);
    }
  };

  const clearTestResults = () => {
    setTestResults([]);
  };

  const clearEPGData = () => {
    clearEPG();
    addTestResult('EPG data cleared');
  };

  return (
    <div style={{ padding: '20px', maxWidth: '1200px', margin: '0 auto' }}>
      <h1>EPG Integration Test</h1>

      <div style={{ marginBottom: '20px', padding: '16px', background: '#f5f5f5', borderRadius: '8px' }}>
        <h3>Test Configuration</h3>

        <div style={{ marginBottom: '12px' }}>
          <label>
            Profile ID:
            <input
              type="text"
              value={profileId}
              onChange={(e) => setProfileId(e.target.value)}
              style={{ marginLeft: '8px', padding: '4px 8px' }}
            />
          </label>
        </div>

        <div style={{ marginBottom: '12px' }}>
          <label>
            Channel ID:
            <input
              type="text"
              value={channelId}
              onChange={(e) => setChannelId(e.target.value)}
              placeholder="Enter channel ID to test EPG"
              style={{ marginLeft: '8px', padding: '4px 8px', width: '200px' }}
            />
          </label>
        </div>

        <div style={{ marginBottom: '12px' }}>
          <label>
            <input
              type="checkbox"
              checked={showFullSchedule}
              onChange={(e) => setShowFullSchedule(e.target.checked)}
              style={{ marginRight: '8px' }}
            />
            Show Full Schedule (instead of Current & Next)
          </label>
        </div>

        <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
          <button onClick={testEPGUtilities} style={{ padding: '8px 16px' }}>
            Test EPG Utilities
          </button>
          <button onClick={clearEPGData} style={{ padding: '8px 16px' }}>
            Clear EPG Data
          </button>
          <button onClick={clearTestResults} style={{ padding: '8px 16px' }}>
            Clear Test Results
          </button>
        </div>
      </div>

      {testResults.length > 0 && (
        <div style={{ marginBottom: '20px', padding: '16px', background: '#e8f5e8', borderRadius: '8px' }}>
          <h3>Test Results</h3>
          <div style={{ fontFamily: 'monospace', fontSize: '12px', maxHeight: '200px', overflow: 'auto' }}>
            {testResults.map((result, index) => (
              <div key={index} style={{ marginBottom: '4px' }}>
                {result}
              </div>
            ))}
          </div>
        </div>
      )}

      {channelId && (
        <div>
          <h3>EPG Display Test</h3>
          <p>
            Testing EPG display for channel ID: <strong>{channelId}</strong>
            {showFullSchedule ? ' (Full Schedule)' : ' (Current & Next)'}
          </p>

          <EPGDisplay
            profileId={profileId}
            channelId={channelId}
            showFullSchedule={showFullSchedule}
            maxPrograms={showFullSchedule ? 5 : undefined}
          />
        </div>
      )}

      {!channelId && (
        <div style={{ padding: '20px', textAlign: 'center', color: '#666' }}>
          <p>Enter a channel ID above to test the EPG display component.</p>
          <p>Note: You'll need a valid Xtream profile and channel ID for the EPG data to load.</p>
        </div>
      )}
    </div>
  );
};

export default EPGTest;