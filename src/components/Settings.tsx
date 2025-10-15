import ContentSyncManager from "./ContentSyncManager";
import { ImageCacheSettings } from "./settings/ImageCacheSettings";
import { PlayerSettings } from "./settings/PlayerSettings";

function Settings() {
  return (
    <div className="settings-layout">
      <ContentSyncManager />
      <PlayerSettings />
      <ImageCacheSettings />
    </div>
  );
}

export default Settings;
