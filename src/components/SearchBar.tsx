import { useEffect, useRef, useState } from "react";
import { useDebounce } from "../hooks/useDebounce";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  debounceDelay?: number;
}

export default function SearchBar({
  value,
  onChange,
  placeholder = "Search...",
  debounceDelay = 300
}: SearchBarProps) {
  const [localValue, setLocalValue] = useState(value);
  const debouncedValue = useDebounce(localValue, debounceDelay);
  const isFirstRender = useRef(true);
  const prevValueRef = useRef(value);

  useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    if (debouncedValue !== prevValueRef.current) {
      onChange(debouncedValue);
      prevValueRef.current = debouncedValue;
    }
  }, [debouncedValue]);

  useEffect(() => {
    if (value !== localValue && value !== debouncedValue) {
      setLocalValue(value);
      prevValueRef.current = value;
    }
  }, [value]);

  const handleClear = () => {
    setLocalValue("");
    onChange("");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.ctrlKey) {
      switch (e.key) {
        case "w":
          e.preventDefault();
          const input = e.currentTarget;
          const cursorPos = input.selectionStart || 0;
          const beforeCursor = localValue.substring(0, cursorPos);
          const afterCursor = localValue.substring(cursorPos);
          const words = beforeCursor.trimEnd();
          const lastSpaceIndex = words.lastIndexOf(" ");
          const newBeforeCursor = lastSpaceIndex >= 0 ? words.substring(0, lastSpaceIndex + 1) : "";
          const newValue = newBeforeCursor + afterCursor;
          setLocalValue(newValue);
          setTimeout(() => {
            input.setSelectionRange(newBeforeCursor.length, newBeforeCursor.length);
          }, 0);
          break;

        case "u":
          e.preventDefault();
          setLocalValue("");
          break;

        case "c":
          e.preventDefault();
          e.currentTarget.blur();
          break;
      }
    }
  };

  return (
    <div className="modern-search-container">
      <div className="search-icon">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.35-4.35" />
        </svg>
      </div>
      <input
        type="text"
        className="modern-search-input"
        placeholder={placeholder}
        value={localValue}
        onChange={(e) => setLocalValue(e.target.value)}
        onKeyDown={handleKeyDown}
      />
      {localValue && (
        <button
          className="modern-clear-btn"
          onClick={handleClear}
          type="button"
          title="Clear search"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      )}
    </div>
  );
}
