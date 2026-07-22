import { useEffect, useRef } from "react";
import { useConsole, type ConsoleLevel } from "../hooks/useConsole";

const LEVEL_ICONS: Record<ConsoleLevel, string> = {
  log: "›",
  info: "ℹ",
  warn: "⚠",
  error: "⛔",
};

export default function ConsoleTab() {
  const { entries, clear } = useConsole();
  const listRef = useRef<HTMLDivElement>(null);
  const shouldAutoScrollRef = useRef(true);

  useEffect(() => {
    const list = listRef.current;
    if (list && shouldAutoScrollRef.current) {
      list.scrollTop = list.scrollHeight;
    }
  }, [entries]);

  const handleScroll = () => {
    const list = listRef.current;
    if (!list) return;
    shouldAutoScrollRef.current =
      list.scrollHeight - list.scrollTop - list.clientHeight < 24;
  };

  return (
    <section className="console-tab" data-testid="bottom-tabpanel-console">
      <button className="bottom-dock-clear" type="button" onClick={clear}>
        Clear
      </button>
      <div className="console-list" ref={listRef} onScroll={handleScroll}>
        {entries.length === 0 ? (
          <p className="bottom-dock-empty">Console output will appear here.</p>
        ) : (
          entries.map((entry) => (
            <div
              className={`console-entry console-entry-${entry.level}`}
              key={entry.id}
            >
              <time dateTime={new Date(entry.ts).toISOString()}>
                {new Date(entry.ts).toLocaleTimeString()}
              </time>
              <span className="console-entry-icon" aria-label={entry.level}>
                {LEVEL_ICONS[entry.level]}
              </span>
              <span className="console-entry-message">{entry.message}</span>
            </div>
          ))
        )}
      </div>
    </section>
  );
}
