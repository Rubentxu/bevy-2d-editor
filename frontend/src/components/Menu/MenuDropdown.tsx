import {
  Children,
  isValidElement,
  useEffect,
  useRef,
  type KeyboardEvent,
} from "react";
import type { MenuDropdownProps } from "../../data/menus";
import MenuItem from "./MenuItem";
import MenuSeparator from "./MenuSeparator";

export default function MenuDropdown({
  label,
  items,
  children,
  open,
  onOpen,
  onClose,
  testId,
}: MenuDropdownProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  const hoverTimer = useRef<number | null>(null);

  useEffect(() => {
    if (!open) return;

    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) onClose();
    };
    const handleEscape = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape") {
        if (hoverTimer.current !== null)
          window.clearTimeout(hoverTimer.current);
        onClose();
      }
    };

    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleEscape);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleEscape);
    };
  }, [open, onClose]);

  useEffect(
    () => () => {
      if (hoverTimer.current !== null) window.clearTimeout(hoverTimer.current);
    },
    [],
  );

  const focusItems = () =>
    Array.from(
      rootRef.current?.querySelectorAll<HTMLButtonElement>(
        '.menu-dropdown [role="menuitem"]:not(:disabled)',
      ) ?? [],
    );

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (
      !open ||
      !["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(event.key)
    ) {
      return;
    }
    event.preventDefault();
    if (event.key === "Escape") {
      if (hoverTimer.current !== null) window.clearTimeout(hoverTimer.current);
      onClose();
      return;
    }

    const focusable = focusItems();
    if (event.key === "Enter") {
      (document.activeElement as HTMLButtonElement | null)?.click();
      return;
    }
    const activeIndex = focusable.indexOf(
      document.activeElement as HTMLButtonElement,
    );
    const nextIndex =
      event.key === "ArrowDown"
        ? (activeIndex + 1 + focusable.length) % focusable.length
        : (activeIndex - 1 + focusable.length) % focusable.length;
    focusable[nextIndex]?.focus();
  };

  const handleMouseEnter = () => {
    if (hoverTimer.current !== null) window.clearTimeout(hoverTimer.current);
    hoverTimer.current = window.setTimeout(onOpen, 200);
  };

  const renderChildren = children
    ? Children.map(children, (child) => child)
    : items?.map((item, index) =>
        item.separator ? (
          <MenuSeparator key={`separator-${index}`} />
        ) : (
          <div className="menu-item-container" key={item.label}>
            <MenuItem
              {...item}
              onClick={
                item.submenu
                  ? undefined
                  : () => {
                      item.onClick?.();
                      onClose();
                    }
              }
            />
            {item.submenu && (
              <div
                className="menu-submenu"
                role="menu"
                aria-label={`${item.label} submenu`}
              >
                {item.submenu.map((subItem) => (
                  <MenuItem
                    key={subItem.label}
                    {...subItem}
                    onClick={() => {
                      subItem.onClick?.();
                      onClose();
                    }}
                  />
                ))}
              </div>
            )}
          </div>
        ),
      );

  return (
    <div
      ref={rootRef}
      className={`menu${open ? " open" : ""}`}
      data-testid={testId}
      data-menu={label.toLowerCase()}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={() => {
        if (hoverTimer.current !== null)
          window.clearTimeout(hoverTimer.current);
      }}
      onKeyDown={handleKeyDown}
    >
      <button
        type="button"
        className="menu-trigger"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => (open ? onClose() : onOpen())}
      >
        {label}
      </button>
      {open && (
        <div className="menu-dropdown" role="menu" aria-label={`${label} menu`}>
          {renderChildren}
        </div>
      )}
    </div>
  );
}
