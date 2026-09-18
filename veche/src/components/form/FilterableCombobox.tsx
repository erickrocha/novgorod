import React, { useState, useEffect, useRef, useMemo } from "react";
import { ChevronDown, Check, X, Loader2 } from "lucide-react";

export interface ComboboxOption {
  value: string;
  label: string;
  sublabel?: string;
  data?: unknown;
}

interface FilterableComboboxProps {
  id?: string;
  value: string;
  options: ComboboxOption[];
  onChange: (value: string, option?: ComboboxOption) => void;
  placeholder?: string;
  disabled?: boolean;
  required?: boolean;
  loading?: boolean;
  emptyText?: string;
  className?: string;
}

export default function FilterableCombobox({
  id,
  value,
  options,
  onChange,
  placeholder = "Select an option...",
  disabled = false,
  required = false,
  loading = false,
  emptyText = "No options found",
  className = "",
}: FilterableComboboxProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Find currently selected option
  const selectedOption = useMemo(() => {
    return options.find(
      (opt) =>
        opt.value.toLowerCase() === value.toLowerCase() ||
        opt.label.toLowerCase() === value.toLowerCase(),
    );
  }, [options, value]);

  // When dropdown opens or value changes, reset filter query to empty so all options show initially
  useEffect(() => {
    if (!isOpen) {
      setQuery("");
      setHighlightedIndex(-1);
    }
  }, [isOpen]);

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };
    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [isOpen]);

  // Filtered options based on query
  const filteredOptions = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return options;
    return options.filter(
      (opt) =>
        opt.label.toLowerCase().includes(q) ||
        opt.value.toLowerCase().includes(q) ||
        (opt.sublabel && opt.sublabel.toLowerCase().includes(q)),
    );
  }, [options, query]);

  const handleSelect = (option: ComboboxOption) => {
    onChange(option.value, option);
    setIsOpen(false);
    setQuery("");
  };

  const handleClear = (e: React.MouseEvent) => {
    e.stopPropagation();
    onChange("");
    setQuery("");
    inputRef.current?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (disabled) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (!isOpen) {
        setIsOpen(true);
        setHighlightedIndex(0);
      } else {
        setHighlightedIndex((prev) =>
          prev < filteredOptions.length - 1 ? prev + 1 : 0,
        );
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (!isOpen) {
        setIsOpen(true);
        setHighlightedIndex(filteredOptions.length - 1);
      } else {
        setHighlightedIndex((prev) =>
          prev > 0 ? prev - 1 : filteredOptions.length - 1,
        );
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (isOpen && highlightedIndex >= 0 && highlightedIndex < filteredOptions.length) {
        handleSelect(filteredOptions[highlightedIndex]);
      } else if (!isOpen) {
        setIsOpen(true);
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      setIsOpen(false);
    } else if (e.key === "Tab") {
      setIsOpen(false);
    }
  };

  const displayValue = isOpen
    ? query
    : selectedOption
    ? selectedOption.label
    : value;

  return (
    <div ref={containerRef} className={`relative w-full ${className}`}>
      {/* Hidden input to support form required validation */}
      {required && (
        <input
          type="text"
          value={value}
          onChange={() => {}}
          required={required}
          className="sr-only"
          tabIndex={-1}
          aria-hidden="true"
        />
      )}

      <div
        className={`relative flex h-11 w-full items-center rounded-lg border transition-colors shadow-theme-xs ${
          disabled
            ? "bg-gray-100 dark:bg-gray-800/50 border-gray-300 dark:border-gray-700 cursor-not-allowed opacity-70"
            : isOpen
            ? "border-brand-500 ring-3 ring-brand-500/10 dark:border-brand-500 bg-transparent"
            : "border-gray-300 dark:border-gray-700 bg-transparent hover:border-gray-400 dark:hover:border-gray-600"
        }`}
        onClick={() => {
          if (!disabled) {
            setIsOpen(true);
            inputRef.current?.focus();
          }
        }}
      >
        <input
          ref={inputRef}
          id={id}
          type="text"
          disabled={disabled}
          placeholder={selectedOption ? selectedOption.label : placeholder}
          value={displayValue}
          onChange={(e) => {
            setQuery(e.target.value);
            if (!isOpen) setIsOpen(true);
            setHighlightedIndex(0);
          }}
          onFocus={() => {
            if (!disabled) setIsOpen(true);
          }}
          onKeyDown={handleKeyDown}
          autoComplete="off"
          className="h-full w-full rounded-lg bg-transparent px-4 py-2.5 text-sm text-gray-800 placeholder:text-gray-400 focus:outline-hidden dark:text-white/90 dark:placeholder:text-white/30"
        />

        <div className="flex items-center gap-1 pe-3 text-gray-400">
          {loading ? (
            <Loader2 size={16} className="animate-spin text-brand-500" />
          ) : (
            <>
              {value && !disabled && (
                <button
                  type="button"
                  tabIndex={-1}
                  onClick={handleClear}
                  className="rounded-full p-1 hover:bg-gray-200 dark:hover:bg-gray-700 hover:text-gray-600 dark:hover:text-gray-200 transition-colors"
                  aria-label="Clear selection"
                >
                  <X size={14} />
                </button>
              )}
              <ChevronDown
                size={16}
                className={`transition-transform duration-200 ${
                  isOpen ? "rotate-180 text-brand-500" : ""
                }`}
              />
            </>
          )}
        </div>
      </div>

      {/* Dropdown Menu */}
      {isOpen && !disabled && (
        <div className="absolute z-50 mt-1 max-h-60 w-full overflow-y-auto rounded-lg border border-gray-200 bg-white py-1 shadow-lg dark:border-gray-700 dark:bg-gray-900">
          {filteredOptions.length === 0 ? (
            <div className="px-4 py-3 text-center text-sm text-gray-500 dark:text-gray-400">
              {loading ? "Loading options..." : emptyText}
            </div>
          ) : (
            filteredOptions.map((opt, idx) => {
              const isSelected =
                opt.value.toLowerCase() === value.toLowerCase() ||
                opt.label.toLowerCase() === value.toLowerCase();
              const isHighlighted = idx === highlightedIndex;

              return (
                <button
                  key={`${opt.value}-${idx}`}
                  type="button"
                  onMouseEnter={() => setHighlightedIndex(idx)}
                  onClick={() => handleSelect(opt)}
                  className={`flex w-full items-center justify-between px-4 py-2 text-start text-sm transition-colors ${
                    isSelected
                      ? "bg-brand-50 font-medium text-brand-600 dark:bg-brand-950/40 dark:text-brand-400"
                      : isHighlighted
                      ? "bg-gray-100 text-gray-900 dark:bg-gray-800 dark:text-white"
                      : "text-gray-700 dark:text-gray-300"
                  }`}
                >
                  <div className="flex flex-col">
                    <span>{opt.label}</span>
                    {opt.sublabel && (
                      <span className="text-xs text-gray-400 dark:text-gray-500">
                        {opt.sublabel}
                      </span>
                    )}
                  </div>
                  {isSelected && <Check size={16} className="shrink-0 text-brand-500" />}
                </button>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}
