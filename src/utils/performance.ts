/**
 * Performance optimization utilities
 * Centralized functions for improving app performance
 */

/**
 * Measure component render time
 * Useful for performance profiling
 */
export function measureRenderTime(componentName: string): {
  start: () => void;
  end: () => void;
} {
  let startTime: number;

  return {
    start: () => {
      startTime = performance.now();
    },
    end: () => {
      const endTime = performance.now();
      const renderTime = endTime - startTime;

      // Only log in development
      if (import.meta.env.DEV) {
        console.log(`[Performance] ${componentName} rendered in ${renderTime.toFixed(2)}ms`);
      }
    },
  };
}

