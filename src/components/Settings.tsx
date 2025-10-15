import { ImageCacheSettings } from "./settings/ImageCacheSettings";
import { PlayerSettings } from "./settings/PlayerSettings";

function Settings() {
  return (
    <div className="settings-layout">
      <PlayerSettings />
      <ImageCacheSettings />
    </div>
  );
}

export default Settings;
