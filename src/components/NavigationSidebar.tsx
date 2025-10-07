import { useUIStore } from "../stores";
import {
  HeartIcon,
  HelpIcon,
  HistoryIcon,
  MovieIcon,
  ProfileIcon,
  SeriesIcon,
  SettingsIcon,
  TvIcon,
  UsersIcon
} from "./Icons";
import ProfileSelector from "./ProfileSelector";

type Tab =
  | "channels"
  | "favorites"
  | "groups"
  | "history"
  | "movies"
  | "series"
  | "profiles"
  | "help"
  | "settings";

export default function NavigationSidebar() {
  const { activeTab, setActiveTab } = useUIStore();

  return (
    <div className="nav-sidebar">
      <div className="app-header">
        <div className="app-logo">
          <img className="app-logo-icon" src="/logo.png" alt="Tollo logo" />
        </div>
        <div className="profile-selector-container">
          <ProfileSelector />
        </div>
        <nav className="nav-menu">
          <button
            className={`nav-button ${activeTab === "channels" ? "active" : ""}`}
            onClick={() => setActiveTab("channels")}
          >
            <TvIcon />
            Channels
          </button>
          <button
            className={`nav-button ${activeTab === "movies" ? "active" : ""}`}
            onClick={() => setActiveTab("movies")}
          >
            <MovieIcon />
            Movies
          </button>
          <button
            className={`nav-button ${activeTab === "series" ? "active" : ""}`}
            onClick={() => setActiveTab("series")}
          >
            <SeriesIcon />
            Series
          </button>
          <button
            className={`nav-button ${activeTab === "favorites" ? "active" : ""}`}
            onClick={() => setActiveTab("favorites")}
          >
            <HeartIcon />
            Favorites
          </button>
          <button
            className={`nav-button ${activeTab === "history" ? "active" : ""}`}
            onClick={() => setActiveTab("history")}
          >
            <HistoryIcon />
            History
          </button>
          <button
            className={`nav-button ${activeTab === "groups" ? "active" : ""}`}
            onClick={() => setActiveTab("groups")}
          >
            <UsersIcon />
            Groups
          </button>
          <button
            className={`nav-button ${activeTab === "profiles" ? "active" : ""}`}
            onClick={() => setActiveTab("profiles")}
          >
            <ProfileIcon />
            Profiles
          </button>
          <button
            className={`nav-button ${activeTab === "help" ? "active" : ""}`}
            onClick={() => setActiveTab("help")}
          >
            <HelpIcon />
            Help
          </button>
          <button
            className={`nav-button ${activeTab === "settings" ? "active" : ""}`}
            onClick={() => setActiveTab("settings")}
          >
            <SettingsIcon />
            Settings
          </button>
        </nav>
      </div>
    </div>
  );
}

export type { Tab };

