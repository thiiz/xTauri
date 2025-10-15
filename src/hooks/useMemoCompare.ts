import { useEffect, useRef } from 'react';

/**
 * Custom hook that memoizes a value based on a comparison function
 * Useful for preventing unnecessary re-renders when objects change reference but not content
 */
export function useMemoCompare<T>(
  value: T,
  compare: (prev: T | undefined, next: T) => boolean
): T {
  const ref = useRef<T>();

  if (!ref.current || !compare(ref.current, value)) {
    ref.current = value;
  }

  return ref.current;
}

/**
 * Hook that returns previous value
 * Useful for comparing current and previous props/state
 */
export function usePrevious<T>(value: T): T | undefined {
  const ref = useRef<T>();

  useEffect(() => {
    ref.current = value;
  }, [value]);

  return ref.current;
}

/**
 * Deep comparison memoization hook
 * Prevents re-renders when object content hasn't changed
 */
export function useDeepCompareMemo<T>(value: T): T {
  return useMemoCompare(value, (prev, next) => {
    return JSON.stringify(prev) === JSON.stringify(next);
  });
}

/**
 * Shallow comparison memoization hook
 * Faster than deep comparison, good for most cases
 */
export function useShallowCompareMemo<T extends Record<string, any>>(value: T): T {
  return useMemoCompare(value, (prev, next) => {
    if (!prev) return false;

    const keys1 = Object.keys(prev);
    const keys2 = Object.keys(next);

    if (keys1.length !== keys2.length) return false;

    for (const key of keys1) {
      if (prev[key] !== next[key]) return false;
    }

    return true;
  });
}
