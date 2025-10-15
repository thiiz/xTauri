import { useCallback, useEffect, useRef, useState } from 'react';

export interface AsyncState<T> {
  data: T | null;
  error: Error | null;
  isLoading: boolean;
  isSuccess: boolean;
  isError: boolean;
}

export interface UseAsyncOptions {
  onSuccess?: (data: any) => void;
  onError?: (error: Error) => void;
}

/**
 * Custom hook for handling async operations with loading, error, and success states
 * Prevents memory leaks and race conditions
 */
export function useAsync<T, Args extends any[] = []>(
  asyncFunction: (...args: Args) => Promise<T>,
  options: UseAsyncOptions = {}
): {
  execute: (...args: Args) => Promise<T | null>;
  reset: () => void;
  state: AsyncState<T>;
} {
  const { onSuccess, onError } = options;

  const [state, setState] = useState<AsyncState<T>>({
    data: null,
    error: null,
    isLoading: false,
    isSuccess: false,
    isError: false,
  });

  // Track if component is mounted to prevent state updates after unmount
  const isMountedRef = useRef(true);
  const pendingPromiseRef = useRef<Promise<T> | null>(null);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const execute = useCallback(
    async (...args: Args): Promise<T | null> => {
      // Set loading state
      if (isMountedRef.current) {
        setState({
          data: null,
          error: null,
          isLoading: true,
          isSuccess: false,
          isError: false,
        });
      }

      try {
        const promise = asyncFunction(...args);
        pendingPromiseRef.current = promise;

        const data = await promise;

        // Only update state if this is still the pending promise and component is mounted
        if (pendingPromiseRef.current === promise && isMountedRef.current) {
          setState({
            data,
            error: null,
            isLoading: false,
            isSuccess: true,
            isError: false,
          });

          onSuccess?.(data);
        }

        return data;
      } catch (error) {
        const err = error instanceof Error ? error : new Error(String(error));

        // Only update state if component is mounted
        if (isMountedRef.current) {
          setState({
            data: null,
            error: err,
            isLoading: false,
            isSuccess: false,
            isError: true,
          });

          onError?.(err);
        }

        return null;
      }
    },
    [asyncFunction, onSuccess, onError]
  );

  const reset = useCallback(() => {
    pendingPromiseRef.current = null;
    if (isMountedRef.current) {
      setState({
        data: null,
        error: null,
        isLoading: false,
        isSuccess: false,
        isError: false,
      });
    }
  }, []);

  return { execute, reset, state };
}

/**
 * Hook for async operations that execute immediately on mount
 * Only use for functions with no required arguments
 */
export function useAsyncImmediate<T>(
  asyncFunction: () => Promise<T>,
  options: UseAsyncOptions = {}
): {
  execute: () => Promise<T | null>;
  reset: () => void;
  state: AsyncState<T>;
} {
  const asyncHook = useAsync(asyncFunction, options);

  useEffect(() => {
    asyncHook.execute();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return asyncHook;
}

/**
 * Hook for handling multiple async operations in parallel
 */
export function useAsyncBatch<T>(
  asyncFunctions: Array<() => Promise<T>>,
  options: UseAsyncOptions = {}
): {
  execute: () => Promise<T[]>;
  reset: () => void;
  state: AsyncState<T[]>;
} {
  const { onSuccess, onError } = options;

  const [state, setState] = useState<AsyncState<T[]>>({
    data: null,
    error: null,
    isLoading: false,
    isSuccess: false,
    isError: false,
  });

  const isMountedRef = useRef(true);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  const execute = useCallback(async (): Promise<T[]> => {
    if (isMountedRef.current) {
      setState({
        data: null,
        error: null,
        isLoading: true,
        isSuccess: false,
        isError: false,
      });
    }

    try {
      const results = await Promise.all(asyncFunctions.map(fn => fn()));

      if (isMountedRef.current) {
        setState({
          data: results,
          error: null,
          isLoading: false,
          isSuccess: true,
          isError: false,
        });

        onSuccess?.(results);
      }

      return results;
    } catch (error) {
      const err = error instanceof Error ? error : new Error(String(error));

      if (isMountedRef.current) {
        setState({
          data: null,
          error: err,
          isLoading: false,
          isSuccess: false,
          isError: true,
        });

        onError?.(err);
      }

      return [];
    }
  }, [asyncFunctions, onSuccess, onError]);

  const reset = useCallback(() => {
    if (isMountedRef.current) {
      setState({
        data: null,
        error: null,
        isLoading: false,
        isSuccess: false,
        isError: false,
      });
    }
  }, []);

  return { execute, reset, state };
}

/**
 * Hook for handling async operations with retry logic
 */
export function useAsyncWithRetry<T, Args extends any[] = []>(
  asyncFunction: (...args: Args) => Promise<T>,
  maxRetries: number = 3,
  retryDelay: number = 1000,
  options: UseAsyncOptions = {}
): {
  execute: (...args: Args) => Promise<T | null>;
  reset: () => void;
  state: AsyncState<T> & { retryCount: number };
} {
  const { onSuccess, onError } = options;
  const [retryCount, setRetryCount] = useState(0);

  const asyncHook = useAsync<T, Args>(asyncFunction, {
    onSuccess,
    onError: (error) => {
      if (retryCount < maxRetries) {
        setTimeout(() => {
          setRetryCount(prev => prev + 1);
        }, retryDelay * Math.pow(2, retryCount)); // Exponential backoff
      } else {
        onError?.(error);
      }
    },
  });

  const execute = useCallback(
    async (...args: Args): Promise<T | null> => {
      setRetryCount(0);
      return asyncHook.execute(...args);
    },
    [asyncHook]
  );

  const reset = useCallback(() => {
    setRetryCount(0);
    asyncHook.reset();
  }, [asyncHook]);

  return {
    execute,
    reset,
    state: { ...asyncHook.state, retryCount },
  };
}
