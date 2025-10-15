# Project Structure

## Root Layout

```
├── src/                    # Frontend React application
├── src-tauri/              # Rust backend (Tauri)
├── public/                 # Static assets
├── dist/                   # Build output (gitignored)
├── node_modules/           # JS dependencies (gitignored)
└── .kiro/                  # Kiro IDE configuration
```

## Frontend Structure (`src/`)

```
src/
├── components/             # React components
│   ├── VirtualMovieGrid.tsx
│   ├── VirtualSeriesBrowser.tsx
│   ├── MainContent.tsx
│   ├── VideoPlayerWrapper.tsx
│   ├── ContentDetails.tsx
│   ├── NavigationSidebar.tsx
│   ├── Settings.tsx
│   ├── ProfileManager.tsx
│   └── Help.tsx
├── hooks/                  # Custom React hooks
│   ├── useDebounce.ts
│   ├── useAsync.ts
│   ├── useIntersectionObserver.ts
│   ├── useMemoCompare.ts
│   ├── useLocalStorage.ts
│   ├── useKeyboardNavigation.ts
│   ├── useContentMetadata.ts
│   └── useContentPlayback.ts
├── stores/                 # Zustand state management
│   ├── searchStore.ts
│   ├── settingsStore.ts
│   └── [other stores]
├── utils/                  # Utility functions
│   └── performance.ts      # Performance utilities (throttle, debounce, cache)
├── constants/              # Application constants
│   └── index.ts            # Centralized constants
├── types/                  # TypeScript type definitions
│   └── types.ts            # Xtream API types
├── styles/                 # CSS files
├── assets/                 # Images, icons, etc.
├── App.tsx                 # Root component
├── App.css                 # Root styles
├── main.tsx                # React entry point
└── vite-env.d.ts           # Vite type definitions
```

## Backend Structure (`src-tauri/`)

```
src-tauri/
├── src/                    # Rust source code
│   └── main.rs             # Entry point
├── benches/                # Performance benchmarks
├── capabilities/           # Tauri capabilities config
├── data/                   # SQLite database location
├── gen/                    # Generated code
├── icons/                  # Application icons
├── target/                 # Rust build output (gitignored)
├── Cargo.toml              # Rust dependencies
├── Cargo.lock              # Rust dependency lock
├── build.rs                # Build script
└── tauri.conf.json         # Tauri configuration
```

## Architecture Patterns

### Component Organization
- Components are organized by feature/purpose
- Virtual scrolling components use `react-virtuoso`
- Video components handle HLS streaming with `hls.js`
- All components use TypeScript with strict typing

### State Management
- Zustand stores for global state (no Redux)
- Each store handles a specific domain (search, settings, UI, channels, etc.)
- Stores use `invoke` from `@tauri-apps/api/core` to call Rust backend
- Local component state with `useState` for UI-only state

### Custom Hooks Pattern
- Hooks are prefixed with `use`
- Located in `src/hooks/` directory
- Reusable logic extracted from components
- Examples: `useDebounce`, `useAsync`, `useIntersectionObserver`

### Performance Optimization
- Virtual scrolling for large lists (movies, series, channels)
- Debounced search queries (300ms default)
- Memoized computations with `useMemo` and `useCallback`
- Deep comparison utilities in `useMemoCompare`
- TTL cache for API responses
- Lazy loading images with Intersection Observer

### Constants Management
- All magic numbers and strings in `src/constants/index.ts`
- Organized by category (DEBOUNCE_DELAY, PAGINATION, SEARCH, etc.)
- Feature flags for gradual rollout
- Error and success messages centralized

### Type Safety
- Comprehensive TypeScript types in `src/types/types.ts`
- Xtream API types fully documented with JSDoc
- Strict mode enabled, no implicit any
- Type inference preferred over explicit types where clear

## Communication Pattern

Frontend ↔ Backend communication uses Tauri's `invoke` API:

```typescript
import { invoke } from "@tauri-apps/api/core";

// Frontend calls Rust function
const channels = await invoke<Channel[]>("get_channels", { id: profileId });
```

## Styling Approach

- CSS files co-located with components
- Global styles in `App.css`
- No CSS-in-JS or styled-components
- Responsive design with CSS Grid and Flexbox

## Documentation Files

- `README.md` - Project overview and setup
- `USAGE_GUIDE.md` - How to use new hooks and utilities
- `REFACTORING_REPORT.md` - Technical details of improvements
- `IMPLEMENTATION_CHECKLIST.md` - Implementation roadmap
- `MIGRATION_GUIDE.md` - Migration instructions
- `USER_GUIDE.md` - End-user documentation
