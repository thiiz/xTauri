# Logger Migration Guide

## Quick Reference

### Import Statement
```typescript
import { logger, storeLogger, apiLogger, hookLogger } from '../utils/logger';
```

### Migration Patterns

#### 1. Simple console.log → logger.debug
```typescript
// Before
console.log('User logged in', userId);

// After
logger.debug('User logged in', userId);
```

#### 2. console.error → logger.error
```typescript
// Before
console.error('Failed to fetch data:', error);

// After
apiLogger.error('Failed to fetch data', error);
```

#### 3. console.warn → logger.warn
```typescript
// Before
console.warn('Cache fetch failed, falling back to API:', error);

// After
apiLogger.warn('Cache fetch failed, falling back to API', error);
```

#### 4. console.info → logger.info
```typescript
// Before
console.info('Settings saved successfully');

// After
logger.info('Settings saved successfully');
```

### Module-Specific Loggers

#### Store Files (src/stores/*.ts)
```typescript
import { storeLogger } from '../utils/logger';

// Use storeLogger for all store-related logging
storeLogger.debug('State updated', newState);
storeLogger.error('Failed to update state', error);
```

#### Hook Files (src/hooks/*.ts)
```typescript
import { hookLogger } from '../utils/logger';

// Use hookLogger for all hook-related logging
hookLogger.debug('Hook initialized', config);
hookLogger.warn('Hook dependency changed', dep);
```

#### API/Service Files
```typescript
import { apiLogger } from '../utils/logger';

// Use apiLogger for all API-related logging
apiLogger.info('API call started', endpoint);
apiLogger.error('API call failed', error);
```

#### Component Files (src/components/*.tsx)
```typescript
import { logger } from '../utils/logger';

// Use general logger for component logging
logger.debug('Component mounted', props);
logger.warn('Invalid prop received', prop);
```

## Files to Migrate

### Priority 1: Store Files (High Impact)
- [ ] `src/stores/xtreamContentStore.ts` (12 instances)
- [ ] `src/stores/searchStore.ts` (4 instances)
- [ ] `src/stores/profileStore.ts` (4 instances)
- [ ] `src/stores/channelStore.ts` (3 instances)

### Priority 2: Hook Files (Medium Impact)
- [ ] `src/hooks/useLocalStorage.ts` (6 instances)
- [ ] `src/hooks/useImageCache.ts` (6 instances)

### Priority 3: Utility Files (Low Impact)
- [ ] `src/utils/epgUtils.ts` (6 instances)

## Example Migration: xtreamContentStore.ts

### Before
```typescript
try {
  const channels = await invoke('get_xtream_channels', { profileId });
  return channels;
} catch (error) {
  console.warn('Cache fetch failed, falling back to API:', error);
  // fallback logic
}
```

### After
```typescript
import { storeLogger } from '../utils/logger';

try {
  const channels = await invoke('get_xtream_channels', { profileId });
  return channels;
} catch (error) {
  storeLogger.warn('Cache fetch failed, falling back to API', error);
  // fallback logic
}
```

## Benefits

1. **Environment Awareness**: Logs only appear in development
2. **Consistent Format**: All logs have timestamps and prefixes
3. **Easy Filtering**: Module-specific loggers help identify issues
4. **Production Ready**: Zero console output in production builds
5. **Better Debugging**: Structured logging makes debugging easier

## Testing After Migration

1. Run in development mode:
   ```bash
   bun dev:tauri
   ```
   - Verify logs appear in console
   - Check log format is correct

2. Build for production:
   ```bash
   bun build:tauri
   ```
   - Verify no console output
   - Check application works correctly

3. Type check:
   ```bash
   bun type-check
   ```
   - Ensure no TypeScript errors

## Notes

- The logger is automatically disabled in production builds
- Use appropriate log levels:
  - `debug`: Development-only information
  - `info`: Important but non-critical information
  - `warn`: Warning conditions that should be addressed
  - `error`: Error conditions that need attention
- Module-specific loggers add context automatically
- All loggers support multiple arguments like console methods
