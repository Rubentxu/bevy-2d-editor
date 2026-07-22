import { useCallback, useEffect, useSyncExternalStore } from "react";

export type ConsoleLevel = "log" | "info" | "warn" | "error";

export interface ConsoleEntry {
  id: number;
  ts: number;
  level: ConsoleLevel;
  message: string;
}

const MAX_ENTRIES = 500;

let entries: ConsoleEntry[] = [];
let nextId = 1;
let patched = false;
const listeners = new Set<() => void>();
const originalConsole: Partial<
  Record<ConsoleLevel, (...args: unknown[]) => void>
> = {};

function emit(): void {
  listeners.forEach((listener) => listener());
}

function formatArgument(value: unknown): string {
  if (typeof value === "string") return value;
  if (value instanceof Error) return value.stack ?? value.message;
  try {
    const serialized = JSON.stringify(value);
    return serialized ?? String(value);
  } catch {
    return String(value);
  }
}

function addEntry(level: ConsoleLevel, message: string): void {
  entries = [
    ...entries,
    { id: nextId++, ts: Date.now(), level, message },
  ].slice(-MAX_ENTRIES);
  emit();
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot(): ConsoleEntry[] {
  return entries;
}

function patchConsole(): void {
  if (patched || typeof window === "undefined") return;
  patched = true;

  (["log", "info", "warn", "error"] as ConsoleLevel[]).forEach((level) => {
    const original = console[level].bind(console);
    originalConsole[level] = original;
    console[level] = (...args: unknown[]) => {
      original(...args);
      addEntry(level, args.map(formatArgument).join(" "));
    };
  });
}

export function useConsole() {
  const consoleEntries = useSyncExternalStore(
    subscribe,
    getSnapshot,
    getSnapshot,
  );

  useEffect(() => {
    patchConsole();
  }, []);

  const add = useCallback((level: ConsoleLevel, message: string) => {
    addEntry(level, message);
  }, []);

  const clear = useCallback(() => {
    entries = [];
    emit();
  }, []);

  return { entries: consoleEntries, add, clear };
}
