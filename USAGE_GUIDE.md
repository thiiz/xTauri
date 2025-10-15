# Guia de Uso - Novos Recursos e Otimizações

Este guia mostra como usar os novos hooks, utilitários e constantes adicionados ao projeto.

## 📚 Hooks Customizados

### useDebounce

Debounce de valores para evitar atualizações excessivas:

```typescript
import { useDebounce } from './hooks/useDebounce';

function SearchComponent() {
  const [searchTerm, setSearchTerm] = useState('');
  const debouncedSearchTerm = useDebounce(searchTerm, 300);

  useEffect(() => {
    // Esta busca só executa 300ms após o usuário parar de digitar
    if (debouncedSearchTerm) {
      performSearch(debouncedSearchTerm);
    }
  }, [debouncedSearchTerm]);

  return <input value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} />;
}
```

### useDebouncedCallback

Debounce de callbacks para handlers de eventos:

```typescript
import { useDebouncedCallback } from './hooks/useDebounce';

function Component() {
  const handleSearch = useDebouncedCallback((query: string) => {
    console.log('Searching for:', query);
    // Perform search
  }, 300);

  return <input onChange={(e) => handleSearch(e.target.value)} />;
}
```

### useAsync

Gerenciamento completo de operações assíncronas:

```typescript
import { useAsync } from './hooks/useAsync';

function MovieDetails({ movieId }: { movieId: string }) {
  const { execute, state } = useAsync(
    async (id: string) => {
      const response = await fetch(`/api/movies/${id}`);
      return response.json();
    },
    {
      onSuccess: (data) => console.log('Movie loaded:', data),
      onError: (error) => console.error('Failed to load movie:', error),
    }
  );

  useEffect(() => {
    execute(movieId);
  }, [movieId, execute]);

  if (state.isLoading) return <div>Loading...</div>;
  if (state.isError) return <div>Error: {state.error?.message}</div>;
  if (state.isSuccess) return <div>{state.data.title}</div>;

  return null;
}
```

### useAsyncImmediate

Para operações que devem executar imediatamente (sem argumentos):

```typescript
import { useAsyncImmediate } from './hooks/useAsync';

function UserProfile() {
  const { state } = useAsyncImmediate(
    async () => {
      const response = await fetch('/api/user/profile');
      return response.json();
    },
    {
      onSuccess: (data) => console.log('Profile loaded:', data),
    }
  );

  if (state.isLoading) return <div>Loading...</div>;
  if (state.isError) return <div>Error: {state.error?.message}</div>;
  return <div>Welcome, {state.data?.name}</div>;
}
```

### useAsyncWithRetry

Operações assíncronas com retry automático:

```typescript
import { useAsyncWithRetry } from './hooks/useAsync';

function Component() {
  const { execute, state } = useAsyncWithRetry(
    async () => {
      const response = await fetch('/api/data');
      if (!response.ok) throw new Error('Failed to fetch');
      return response.json();
    },
    3, // max retries
    1000, // retry delay (com exponential backoff)
    {
      onSuccess: (data) => console.log('Success after', state.retryCount, 'retries'),
      onError: (error) => console.error('Failed after all retries:', error),
    }
  );

  return (
    <div>
      <button onClick={() => execute()}>Fetch Data</button>
      {state.isLoading && <div>Loading... (Attempt {state.retryCount + 1})</div>}
      {state.isError && <div>Error: {state.error?.message}</div>}
    </div>
  );
}
```

### useIntersectionObserver

Detecção de visibilidade para lazy loading:

```typescript
import { useIntersectionObserver } from './hooks/useIntersectionObserver';

function LazyImage({ src, alt }: { src: string; alt: string }) {
  const [ref, isVisible] = useIntersectionObserver({
    threshold: 0.1,
    rootMargin: '200px',
    freezeOnceVisible: true, // Para de observar após ficar visível
  });

  return (
    <div ref={ref}>
      {isVisible ? (
        <img src={src} alt={alt} />
      ) : (
        <div className="placeholder">Loading...</div>
      )}
    </div>
  );
}
```

### useMemoCompare

Memoização com comparação customizada:

```typescript
import { useMemoCompare, useDeepCompareMemo } from './hooks/useMemoCompare';

function Component({ data }: { data: ComplexObject }) {
  // Evita re-renders quando o objeto muda de referência mas não de conteúdo
  const memoizedData = useDeepCompareMemo(data);

  // Ou com comparação customizada
  const customMemoized = useMemoCompare(data, (prev, next) => {
    return prev?.id === next?.id && prev?.version === next?.version;
  });

  return <ExpensiveComponent data={memoizedData} />;
}
```

### useLocalStorage

Persistência automática no localStorage:

```typescript
import { useLocalStorage } from './hooks/useLocalStorage';

function Settings() {
  const [volume, setVolume, removeVolume] = useLocalStorage('volume', 1.0);
  const [theme, setTheme] = useLocalStorage('theme', 'dark');

  return (
    <div>
      <input
        type="range"
        min="0"
        max="1"
        step="0.1"
        value={volume}
        onChange={(e) => setVolume(parseFloat(e.target.value))}
      />
      <button onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
        Toggle Theme
      </button>
      <button onClick={removeVolume}>Reset Volume</button>
    </div>
  );
}
```

## 🛠️ Utilitários de Performance

### Throttle e Debounce

```typescript
import { throttle, debounce } from './utils/performance';

// Throttle para scroll handlers
const handleScroll = throttle(() => {
  console.log('Scroll position:', window.scrollY);
}, 100);

window.addEventListener('scroll', handleScroll);

// Debounce para resize handlers
const handleResize = debounce(() => {
  console.log('Window size:', window.innerWidth, window.innerHeight);
}, 200);

window.addEventListener('resize', handleResize);
```

### Memoize

Cache de resultados de funções custosas:

```typescript
import { memoize } from './utils/performance';

const expensiveCalculation = memoize((n: number) => {
  console.log('Calculating...');
  return n * n * n;
});

console.log(expensiveCalculation(5)); // Calcula
console.log(expensiveCalculation(5)); // Retorna do cache
```

### TTLCache

Cache com expiração automática:

```typescript
import { TTLCache } from './utils/performance';

const apiCache = new TTLCache<string, any>(5 * 60 * 1000); // 5 minutos

async function fetchData(url: string) {
  // Verifica cache primeiro
  if (apiCache.has(url)) {
    return apiCache.get(url);
  }

  // Busca dados
  const response = await fetch(url);
  const data = await response.json();

  // Armazena no cache
  apiCache.set(url, data);

  return data;
}
```

### Comparação de Objetos

```typescript
import { deepEqual, shallowEqual } from './utils/performance';

const obj1 = { a: 1, b: { c: 2 } };
const obj2 = { a: 1, b: { c: 2 } };

console.log(deepEqual(obj1, obj2)); // true
console.log(shallowEqual(obj1, obj2)); // false (referências diferentes)
```

### Request Idle Callback

Adiar trabalho não-crítico:

```typescript
import { requestIdleCallback, cancelIdleCallback } from './utils/performance';

const id = requestIdleCallback(() => {
  // Trabalho não-crítico que pode ser adiado
  console.log('Processing in idle time...');
  processLargeDataset();
}, { timeout: 2000 });

// Cancelar se necessário
cancelIdleCallback(id);
```

## 📋 Constantes

### Usando Constantes

```typescript
import { DEBOUNCE_DELAY, PAGINATION, SEARCH, STORAGE_KEYS } from './constants';

// Debounce delays
const searchDebounce = DEBOUNCE_DELAY.SEARCH; // 300ms

// Paginação
const itemsPerPage = PAGINATION.ITEMS_PER_PAGE; // 50
const moviesPerRow = PAGINATION.MOVIES_PER_ROW; // 6

// Busca
if (query.length < SEARCH.MIN_QUERY_LENGTH) {
  return; // Mínimo 2 caracteres
}

// LocalStorage
const volume = localStorage.getItem(STORAGE_KEYS.VOLUME);
```

### Feature Flags

```typescript
import { FEATURES } from './constants';

function Component() {
  return (
    <div>
      {FEATURES.ENABLE_EPG && <EPGComponent />}
      {FEATURES.ENABLE_FAVORITES && <FavoritesButton />}
      {FEATURES.ENABLE_OFFLINE_MODE && <OfflineIndicator />}
    </div>
  );
}
```

### Mensagens de Erro

```typescript
import { ERROR_MESSAGES, SUCCESS_MESSAGES } from './constants';

try {
  await saveProfile();
  showToast(SUCCESS_MESSAGES.PROFILE_CREATED);
} catch (error) {
  if (error.code === 'NETWORK_ERROR') {
    showToast(ERROR_MESSAGES.NETWORK_ERROR);
  } else {
    showToast(ERROR_MESSAGES.UNKNOWN);
  }
}
```

## 🎯 Exemplos Práticos

### Componente de Busca Otimizado

```typescript
import { useState } from 'react';
import { useDebounce } from './hooks/useDebounce';
import { useAsync } from './hooks/useAsync';
import { SEARCH, DEBOUNCE_DELAY } from './constants';

function SearchComponent() {
  const [query, setQuery] = useState('');
  const debouncedQuery = useDebounce(query, DEBOUNCE_DELAY.SEARCH);

  const { execute, state } = useAsync(async (searchQuery: string) => {
    if (searchQuery.length < SEARCH.MIN_QUERY_LENGTH) {
      return [];
    }
    const response = await fetch(`/api/search?q=${searchQuery}`);
    return response.json();
  });

  useEffect(() => {
    if (debouncedQuery) {
      execute(debouncedQuery);
    }
  }, [debouncedQuery, execute]);

  return (
    <div>
      <input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={`Min ${SEARCH.MIN_QUERY_LENGTH} characters`}
      />
      {state.isLoading && <div>Searching...</div>}
      {state.isError && <div>Error: {state.error?.message}</div>}
      {state.isSuccess && (
        <ul>
          {state.data?.map((item) => (
            <li key={item.id}>{item.name}</li>
          ))}
        </ul>
      )}
    </div>
  );
}
```

### Grid Virtual com Lazy Loading

```typescript
import { Virtuoso } from 'react-virtuoso';
import { useIntersectionObserver } from './hooks/useIntersectionObserver';
import { PAGINATION, IMAGE } from './constants';

function VirtualGrid({ items }: { items: Movie[] }) {
  const rowRenderer = useCallback((index: number) => {
    const startIdx = index * PAGINATION.MOVIES_PER_ROW;
    const endIdx = Math.min(startIdx + PAGINATION.MOVIES_PER_ROW, items.length);
    const rowItems = items.slice(startIdx, endIdx);

    return (
      <div className="row">
        {rowItems.map((item) => (
          <MovieCard key={item.id} movie={item} />
        ))}
      </div>
    );
  }, [items]);

  const totalRows = Math.ceil(items.length / PAGINATION.MOVIES_PER_ROW);

  return (
    <Virtuoso
      totalCount={totalRows}
      itemContent={rowRenderer}
      overscan={PAGINATION.OVERSCAN_COUNT}
    />
  );
}

function MovieCard({ movie }: { movie: Movie }) {
  const [ref, isVisible] = useIntersectionObserver({
    rootMargin: IMAGE.LAZY_LOAD_ROOT_MARGIN,
    freezeOnceVisible: true,
  });

  return (
    <div ref={ref}>
      {isVisible ? (
        <img src={movie.poster} alt={movie.title} />
      ) : (
        <div className="placeholder" />
      )}
      <h3>{movie.title}</h3>
    </div>
  );
}
```

### Settings com Persistência

```typescript
import { useLocalStorage } from './hooks/useLocalStorage';
import { STORAGE_KEYS, VIDEO_PLAYER } from './constants';

function VideoSettings() {
  const [volume, setVolume] = useLocalStorage(
    STORAGE_KEYS.VOLUME,
    VIDEO_PLAYER.DEFAULT_VOLUME
  );
  const [muted, setMuted] = useLocalStorage(STORAGE_KEYS.MUTED, false);
  const [playbackRate, setPlaybackRate] = useLocalStorage(
    STORAGE_KEYS.PLAYBACK_RATE,
    1.0
  );

  return (
    <div>
      <label>
        Volume:
        <input
          type="range"
          min="0"
          max="1"
          step={VIDEO_PLAYER.VOLUME_STEP}
          value={volume}
          onChange={(e) => setVolume(parseFloat(e.target.value))}
        />
      </label>
      <label>
        <input
          type="checkbox"
          checked={muted}
          onChange={(e) => setMuted(e.target.checked)}
        />
        Muted
      </label>
      <label>
        Playback Speed:
        <select
          value={playbackRate}
          onChange={(e) => setPlaybackRate(parseFloat(e.target.value))}
        >
          <option value="0.5">0.5x</option>
          <option value="1.0">1.0x</option>
          <option value="1.5">1.5x</option>
          <option value="2.0">2.0x</option>
        </select>
      </label>
    </div>
  );
}
```

## 🚀 Dicas de Performance

1. **Use `useDebounce` para inputs de busca** - Reduz chamadas de API
2. **Use `useAsync` para operações assíncronas** - Previne memory leaks
3. **Use `useIntersectionObserver` para lazy loading** - Melhora tempo de carregamento inicial
4. **Use `useMemoCompare` para props complexas** - Evita re-renders desnecessários
5. **Use `TTLCache` para API responses** - Reduz chamadas de rede
6. **Use constantes ao invés de magic numbers** - Facilita manutenção
7. **Use feature flags para rollout gradual** - Testa features com segurança

## 📖 Referências

- [React Hooks Documentation](https://react.dev/reference/react)
- [Intersection Observer API](https://developer.mozilla.org/en-US/docs/Web/API/Intersection_Observer_API)
- [Web Performance Best Practices](https://web.dev/performance/)
- [React Performance Optimization](https://react.dev/learn/render-and-commit)

---

**Última Atualização**: 2025-01-15
