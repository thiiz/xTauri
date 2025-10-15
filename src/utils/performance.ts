/**
 * Performance optimization utilities
 * Centralized functions for improving app performance
 */

/**
 * Throttle function to limit execution rate
 * Useful for scroll handlers, resize handlers, etc.
 */
export function throttle<T extends (...args: any[]) => any>(
  func: T,
  limit: number
): (...args: Parameters<T>) => void {
  let inThrottle: boolean;
  let lastResult: ReturnType<T>;

  return function (this: any, ...args: Parameters<T>): void {
    if (!inThrottle) {
      inThrottle = true;
      lastResult = func.apply(this, args);
      setTimeout(() => (inThrottle = false), limit);
    }
  };
}

/**
 * Debounce function to delay execution
 * Useful for search inputs, form validation, etc.
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return function (this: any, ...args: Parameters<T>): void {
    const later = () => {
      timeout = null;
      func.apply(this, args);
    };

    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

/**
 * Memoize function results to avoid expensive recalculations
 * Simple memoization for single-argument functions
 */
export function memoize<T extends (...args: any[]) => any>(
  func: T
): T {
  const cache = new Map<string, ReturnType<T>>();

  return ((...args: Parameters<T>): ReturnType<T> => {
    const key = JSON.stringify(args);

    if (cache.has(key)) {
      return cache.get(key)!;
    }

    const result = func(...args);
    cache.set(key, result);
    return result;
  }) as T;
}

/**
 * Deep equality check for objects
 * Useful for preventing unnecessary re-renders
 */
export function deepEqual(obj1: any, obj2: any): boolean {
  if (obj1 === obj2) return true;

  if (
    typeof obj1 !== 'object' ||
    typeof obj2 !== 'object' ||
    obj1 === null ||
    obj2 === null
  ) {
    return false;
  }

  const keys1 = Object.keys(obj1);
  const keys2 = Object.keys(obj2);

  if (keys1.length !== keys2.length) return false;

  for (const key of keys1) {
    if (!keys2.includes(key) || !deepEqual(obj1[key], obj2[key])) {
      return false;
    }
  }

  return true;
}

/**
 * Shallow equality check for objects
 * Faster than deep equality, useful for most cases
 */
export function shallowEqual(obj1: any, obj2: any): boolean {
  if (obj1 === obj2) return true;

  if (
    typeof obj1 !== 'object' ||
    typeof obj2 !== 'object' ||
    obj1 === null ||
    obj2 === null
  ) {
    return false;
  }

  const keys1 = Object.keys(obj1);
  const keys2 = Object.keys(obj2);

  if (keys1.length !== keys2.length) return false;

  for (const key of keys1) {
    if (obj1[key] !== obj2[key]) return false;
  }

  return true;
}

/**
 * Request idle callback wrapper with fallback
 * Useful for deferring non-critical work
 */
export function requestIdleCallback(
  callback: () => void,
  options?: { timeout?: number }
): number {
  if ('requestIdleCallback' in window) {
    return window.requestIdleCallback(callback, options);
  }

  // Fallback for browsers that don't support requestIdleCallback
  return setTimeout(callback, 1) as unknown as number;
}

/**
 * Cancel idle callback wrapper with fallback
 */
export function cancelIdleCallback(id: number): void {
  if ('cancelIdleCallback' in window) {
    window.cancelIdleCallback(id);
  } else {
    clearTimeout(id);
  }
}

/**
 * Batch multiple state updates to reduce re-renders
 * Useful when updating multiple related state values
 */
export function batchUpdates<T>(updates: Array<() => void>): void {
  // React 18+ automatically batches updates
  // This is a placeholder for potential future optimizations
  updates.forEach(update => update());
}

/**
 * Lazy load a component with error boundary
 * Useful for code splitting and reducing initial bundle size
 */
export function lazyWithRetry<T extends React.ComponentType<any>>(
  componentImport: () => Promise<{ default: T }>,
  retries: number = 3
): React.LazyExoticComponent<T> {
  return React.lazy(async () => {
    for (let i = 0; i < retries; i++) {
      try {
        return await componentImport();
      } catch (error) {
        if (i === retries - 1) throw error;
        // Wait before retrying (exponential backoff)
        await new Promise(resolve => setTimeout(resolve, 1000 * Math.pow(2, i)));
      }
    }
    throw new Error('Failed to load component after retries');
  });
}

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

/**
 * Create a cache with TTL (Time To Live)
 * Useful for caching API responses
 */
export class TTLCache<K, V> {
  private cache = new Map<K, { value: V; expiry: number }>();
  private ttl: number;

  constructor(ttlMs: number = 5 * 60 * 1000) {
    this.ttl = ttlMs;
  }

  set(key: K, value: V): void {
    const expiry = Date.now() + this.ttl;
    this.cache.set(key, { value, expiry });
  }

  get(key: K): V | undefined {
    const item = this.cache.get(key);

    if (!item) return undefined;

    if (Date.now() > item.expiry) {
      this.cache.delete(key);
      return undefined;
    }

    return item.value;
  }

  has(key: K): boolean {
    return this.get(key) !== undefined;
  }

  delete(key: K): boolean {
    return this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    // Clean up expired entries first
    for (const [key, item] of this.cache.entries()) {
      if (Date.now() > item.expiry) {
        this.cache.delete(key);
      }
    }
    return this.cache.size;
  }
}

// Import React for lazy loading
import React from 'react';

