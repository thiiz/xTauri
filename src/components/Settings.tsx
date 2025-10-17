import ContentSyncManager from "./ContentSyncManager";
import { PlayerSettings } from "./settings/PlayerSettings";

function Settings() {
  return (
    <div className="settings-layout">
      <ContentSyncManager />
      <PlayerSettings />
    </div>
  );
}

export default Settings;
