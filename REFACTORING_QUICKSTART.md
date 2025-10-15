# Refactoring Quick Start Guide

## What Changed?

The xTauri codebase has been refactored for better performance, code quality, and maintainability. Here's what you need to know to get started.

## 🚀 Quick Start (5 minutes)

### 1. Verify Everything Works
```bash
# Type check
bun type-check

# Run in development
bun dev:tauri

# Build for production
bun build:tauri
```

### 2. Key Changes to Know

#### Logging
**Old way:**
```typescript
console.log('Something happened', data);
console.error('Error occurred', error);
```

**New way:**
```typescript
import { logger } from '@/utils/logger';

logger.debug('Something happened', data);
logger.error('Error occurred', error);
```

#### Error Handling
**New feature:**
```typescript
import ErrorBoundary from '@/components/ErrorBoundary';

<ErrorBoundary>
  <YourComponent />
</ErrorBoundary>
```

#### Performance Monitoring
**New feature:**
```typescript
import { usePerformanceMonitor } from '@/hooks/usePerformanceMonitor';

function MyComponent() {
  usePerformanceMonitor('MyComponent');
  return <div>Content</div>;
}
```

## 📚 Essential Reading

1. **[REFACTORING_SUMMARY.md](./REFACTORING_SUMMARY.md)** - Overview of all changes
2. **[scripts/migrate-to-logger.md](./scripts/migrate-to-logger.md)** - How to use the new logger
3. **[REFACTORING_IMPROVEMENTS.md](./REFACTORING_IMPROVEMENTS.md)** - Detailed technical changes

## 🔧 Common Tasks

### Adding Logging to Your Code

```typescript
// At the top of your file
import { logger, storeLogger, apiLogger, hookLogger } from '@/utils/logger';

// In components
logger.info('Component action', data);

// In stores
storeLogger.debug('State updated', newState);

// In API calls
apiLogger.error('API failed', error);

// In hooks
hookLogger.warn('Hook warning', context);
```

### Adding Error Boundaries

```typescript
// Wrap sections that might error
<ErrorBoundary fallback={<CustomErrorUI />}>
  <RiskyComponent />
</ErrorBoundary>

// With custom error handler
<ErrorBoundary onError={(error, info) => {
  // Custom error handling
}}>
  <Component />
</ErrorBoundary>
```

### Monitoring Performance

```typescript
// Automatic monitoring
function MyComponent() {
  usePerformanceMonitor('MyComponent');
  // Component code
}

// Lifecycle tracking
function MyComponent() {
  useLifecycleLogger('MyComponent',
    () => console.log('Mounted'),
    () => console.log('Unmounted')
  );
  // Component code
}

// Effect measurement
function MyComponent() {
  useMeasuredEffect('fetchData', async () => {
    await fetchData();
  }, [userId]);
  // Component code
}
```

## 🎯 What to Do Next

### If You're Working on Existing Code
1. Replace console statements with logger
2. Add error boundaries around risky sections
3. Add performance monitoring if needed
4. Test your changes

### If You're Adding New Code
1. Use logger instead of console
2. Wrap components in error boundaries
3. Add performance monitoring for complex components
4. Follow the existing patterns

### If You're Fixing Bugs
1. Check the logger output in development
2. Use error boundaries to catch errors
3. Use performance monitoring to find bottlenecks
4. Document your findings

## 🐛 Troubleshooting

### "Logger is not defined"
```typescript
// Add import at top of file
import { logger } from '@/utils/logger';
```

### "ErrorBoundary is not defined"
```typescript
// Add import at top of file
import ErrorBoundary from '@/components/ErrorBoundary';
```

### "No logs appearing in console"
- Logs only appear in development mode
- Check that `import.meta.env.DEV` is true
- Verify logger is imported correctly

### "TypeScript errors after refactoring"
```bash
# Run type check to see errors
bun type-check

# Most common fix: update imports
import { logger } from '@/utils/logger';
```

## 📖 Code Examples

### Complete Component Example
```typescript
import { useCallback, useEffect, useState } from 'react';
import { logger } from '@/utils/logger';
import { usePerformanceMonitor } from '@/hooks/usePerformanceMonitor';
import ErrorBoundary from '@/components/ErrorBoundary';

function MyComponent() {
  // Performance monitoring
  usePerformanceMonitor('MyComponent');
  
  const [data, setData] = useState(null);
  
  // Fetch data with logging
  useEffect(() => {
    const fetchData = async () => {
      try {
        logger.debug('Fetching data');
        const result = await api.getData();
        setData(result);
        logger.info('Data fetched successfully');
      } catch (error) {
        logger.error('Failed to fetch data', error);
      }
    };
    
    fetchData();
  }, []);
  
  const handleClick = useCallback(() => {
    logger.debug('Button clicked', { data });
    // Handle click
  }, [data]);
  
  return (
    <div>
      <button onClick={handleClick}>Click Me</button>
      {data && <div>{data}</div>}
    </div>
  );
}

// Wrap with error boundary
export default function MyComponentWithBoundary() {
  return (
    <ErrorBoundary>
      <MyComponent />
    </ErrorBoundary>
  );
}
```

### Complete Store Example
```typescript
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { storeLogger } from '@/utils/logger';

interface MyStore {
  data: any[];
  loading: boolean;
  error: string | null;
  fetchData: () => Promise<void>;
}

export const useMyStore = create<MyStore>((set, get) => ({
  data: [],
  loading: false,
  error: null,
  
  fetchData: async () => {
    set({ loading: true, error: null });
    storeLogger.debug('Fetching data');
    
    try {
      const data = await invoke('get_data');
      set({ data, loading: false });
      storeLogger.info('Data fetched successfully', { count: data.length });
    } catch (error) {
      storeLogger.error('Failed to fetch data', error);
      set({ error: error as string, loading: false });
    }
  },
}));
```

## 🎓 Learning Resources

### Internal Documentation
- [REFACTORING_SUMMARY.md](./REFACTORING_SUMMARY.md) - Complete overview
- [REFACTORING_IMPROVEMENTS.md](./REFACTORING_IMPROVEMENTS.md) - Technical details
- [scripts/migrate-to-logger.md](./scripts/migrate-to-logger.md) - Logger guide
- [REFACTORING_CHECKLIST.md](./REFACTORING_CHECKLIST.md) - Implementation tasks

### Code Locations
- Logger: `src/utils/logger.ts`
- Error Boundary: `src/components/ErrorBoundary.tsx`
- Performance Hooks: `src/hooks/usePerformanceMonitor.ts`
- Formatters: `src/utils/formatters.ts`
- Performance Utils: `src/utils/performance.ts`

## ⚡ Performance Tips

1. **Use logger instead of console** - Zero overhead in production
2. **Add error boundaries** - Prevent full app crashes
3. **Monitor performance** - Identify slow components
4. **Use memoization wisely** - Don't over-optimize
5. **Test in production mode** - Verify optimizations work

## 🤝 Getting Help

1. **Check the documentation** - Most answers are there
2. **Look at examples** - See how it's used in existing code
3. **Test in development** - Logs and errors are visible
4. **Ask questions** - Better to ask than break things

## ✅ Checklist for New Code

- [ ] Use logger instead of console
- [ ] Add error boundary if component can error
- [ ] Add performance monitoring if complex
- [ ] Use existing formatters for data
- [ ] Follow TypeScript best practices
- [ ] Test in development mode
- [ ] Verify production build works

## 🎉 You're Ready!

You now know the basics of the refactored codebase. Start coding and refer back to this guide as needed.

**Remember:**
- Logger for all logging
- Error boundaries for error handling
- Performance hooks for monitoring
- Test everything in development first

---

**Questions?** Check the detailed documentation or ask for help!
