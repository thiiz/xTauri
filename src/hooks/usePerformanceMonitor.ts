import { useEffect, useRef } from 'react';
import { measureRenderTime } from '../utils/performance';

/**
 * Hook to monitor component render performance
 * Only active in development mode
 * 
 * @param componentName - Name of the component to monitor
 * @param enabled - Whether monitoring is enabled (default: true in dev)
 * 
 * @example
 * function MyComponent() {
 *   usePerformanceMonitor('MyComponent');
 *   return <div>Content</div>;
 * }
 */
export function usePerformanceMonitor(
  componentName: string,
  enabled: boolean = import.meta.env.DEV
): void {
  const timerRef = useRef(measureRenderTime(componentName));
  const renderCountRef = useRef(0);

  useEffect(() => {
    if (!enabled) return;

    renderCountRef.current += 1;
    timerRef.current.start();

    return () => {
      timerRef.current.end();
    };
  });
}

/**
 * Hook to track component mount/unmount lifecycle
 * Useful for debugging memory leaks and lifecycle issues
 * 
 * @param componentName - Name of the component to track
 * @param onMount - Optional callback when component mounts
 * @param onUnmount - Optional callback when component unmounts
 * 
 * @example
 * function MyComponent() {
 *   useLifecycleLogger('MyComponent', 
 *     () => console.log('Mounted'),
 *     () => console.log('Unmounted')
 *   );
 *   return <div>Content</div>;
 * }
 */
export function useLifecycleLogger(
  componentName: string,
  onMount?: () => void,
  onUnmount?: () => void
): void {
  useEffect(() => {
    if (import.meta.env.DEV) {
      console.log(`[Lifecycle] ${componentName} mounted`);
    }
    onMount?.();

    return () => {
      if (import.meta.env.DEV) {
        console.log(`[Lifecycle] ${componentName} unmounted`);
      }
      onUnmount?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
}

/**
 * Hook to measure and log effect execution time
 * Useful for identifying slow effects
 * 
 * @param effectName - Name of the effect to measure
 * @param effect - Effect function to measure
 * @param deps - Effect dependencies
 * 
 * @example
 * useMeasuredEffect('fetchData', async () => {
 *   await fetchData();
 * }, [userId]);
 */
export function useMeasuredEffect(
  effectName: string,
  effect: () => void | (() => void),
  deps: React.DependencyList
): void {
  useEffect(() => {
    if (!import.meta.env.DEV) {
      return effect();
    }

    const startTime = performance.now();
    const cleanup = effect();
    const endTime = performance.now();

    console.log(
      `[Effect] ${effectName} executed in ${(endTime - startTime).toFixed(2)}ms`
    );

    return cleanup;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);
}
