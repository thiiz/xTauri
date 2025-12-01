import { useUIStore } from "../stores";
import {
  HeartIcon,
  HelpIcon,
  HistoryIcon,
  MovieIcon,
  SeriesIcon,
  SettingsIcon,
  TvIcon,
  UsersIcon,
} from "./Icons";
import ProfileSelector from "./ProfileSelector";

type Tab =
  | "channels"
  | "favorites"
  | "groups"
  | "history"
  | "movies"
  | "series"
  | "help"
  | "settings";

const mainNavItems = [
  { id: "channels" as Tab, icon: TvIcon, label: "Channels" },
  { id: "movies" as Tab, icon: MovieIcon, label: "Movies" },
  { id: "series" as Tab, icon: SeriesIcon, label: "Series" },
];

const libraryNavItems = [
  { id: "favorites" as Tab, icon: HeartIcon, label: "Favorites" },
  { id: "history" as Tab, icon: HistoryIcon, label: "History" },
  { id: "groups" as Tab, icon: UsersIcon, label: "Groups" },
];

const settingsNavItems = [
  { id: "help" as Tab, icon: HelpIcon, label: "Help" },
  { id: "settings" as Tab, icon: SettingsIcon, label: "Settings" },
];

export default function NavigationSidebar() {
  const { activeTab, setActiveTab } = useUIStore();

  const renderNavButton = (item: { id: Tab; icon: any; label: string }) => {
    const Icon = item.icon;
    return (
      <button
        key={item.id}
        className={`nav-button ${activeTab === item.id ? "active" : ""}`}
        onClick={() => setActiveTab(item.id)}
      >
        <Icon />
        <span className="nav-button-label">{item.label}</span>
      </button>
    );
  };

  return (
    <div className="nav-sidebar">
      <div className="nav-sidebar-content">
        <div className="nav-header">
          <div className="app-brand">
            <div className="app-brand-icon">📺</div>
            <h1 className="app-brand-title">xTauri</h1>
          </div>
        </div>

        <nav className="nav-menu">
          <div className="nav-section">
            <div className="nav-section-title">Browse</div>
            <div className="nav-section-items">
              {mainNavItems.map(renderNavButton)}
            </div>
          </div>

          <div className="nav-section">
            <div className="nav-section-title">Library</div>
            <div className="nav-section-items">
              {libraryNavItems.map(renderNavButton)}
            </div>
          </div>

          <div className="nav-section nav-section-bottom">
            <div className="nav-section-items">
              {settingsNavItems.map(renderNavButton)}
            </div>
          </div>
        </nav>
      </div>
    </div>
  );
}

export type { Tab };
