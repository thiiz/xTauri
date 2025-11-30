import { useEffect } from "react";
import "./App.css";
import AuthPage from "./pages/AuthPage";
import HomePage from "./pages/HomePage";
import { useProfileStore, useSettingsStore } from "./stores";

function App() {
  const { activeProfile } = useProfileStore();
  const {
    fetchEnablePreview,
    fetchAutoplay,
    fetchMuteOnStart,
    fetchShowControls,
    fetchCacheDuration,
    fetchVolume,
    fetchIsMuted,
  } = useSettingsStore();

  // Fetch all settings on app load - memoized to run only once
  useEffect(() => {
    const loadSettings = async () => {
      try {
        await Promise.all([
          fetchEnablePreview(),
          fetchAutoplay(),
          fetchMuteOnStart(),
          fetchShowControls(),
          fetchCacheDuration(),
          fetchVolume(),
          fetchIsMuted(),
        ]);
      } catch (error) {
        console.error("Failed to load settings:", error);
      }
    };

    loadSettings();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Run only once on mount

  // Routing logic
  if (!activeProfile) {
    return <AuthPage />;
  }

  return <HomePage />;
}

export default App;
