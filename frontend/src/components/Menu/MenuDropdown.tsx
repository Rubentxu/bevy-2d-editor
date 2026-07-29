import {
  Children,
  useEffect,
  useRef,
  useCallback,
  type KeyboardEvent,
} from "react";
import { createPortal } from "react-dom";
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
  anchorRect,
}: MenuDropdownProps) {
  const rootRef = useRef<HTMLDivElement>(null);
  /** Ref to the portaled dropdown container so focus queries work correctly. */
  const portalRef = useRef<HTMLDivElement | null>(null);
  const hoverTimer = useRef<number | null>(null);

  /**
   * Move focus into the dropdown when it opens so keyboard navigation starts
   * on the first item rather than staying on the trigger button.
   */
  useEffect(() => {
    if (!open) return;
    // Defer to the next tick so the portal has already mounted.
    const raf = requestAnimationFrame(() => {
      const items = focusItems();
      if (items.length > 0) {
        items[0].focus();
      }
    });
    return () => cancelAnimationFrame(raf);
  }, [open]);

  /**
   * Outside-click: close when the user clicks anything outside the portaled
   * dropdown AND outside the trigger button. We guard on the trigger element
   * explicitly because the portal renders to document.body, so it is not a
   * descendant of the .menu element in DOM terms.
   */
  useEffect(() => {
    if (!open) return;

    const handlePointerDown = (event: MouseEvent) => {
      // Ignore clicks on any menu trigger button (not just this menu's trigger),
      // so clicking a different menu's trigger opens that menu instead of closing this one.
      if ((event.target as Element)?.closest(".menu-trigger")) return;
      // Clicking inside the portaled dropdown must NOT close the menu.
      if (portalRef.current?.contains(event.target as Node)) return;
      // Any other click outside the menu root closes the menu.
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

  /**
   * Query all keyboard-navigable menu items from the portaled dropdown.
   * Uses the stable data-testid on the portal target rather than DOM hierarchy.
   */
  const focusItems = useCallback((): HTMLButtonElement[] => {
    const portaledDropdown =
      typeof document !== "undefined"
        ? document.body.querySelector<HTMLDivElement>(
            '[data-testid="menu-dropdown"]',
          )
        : null;
    const container =
      portaledDropdown ??
      rootRef.current?.querySelector<HTMLDivElement>(".menu-dropdown");
    if (!container) return [];
    return Array.from(
      container.querySelectorAll<HTMLButtonElement>(
        '[role="menuitem"]:not(:disabled)',
      ) ?? [],
    );
  }, []);

  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (
      !open ||
      !["ArrowDown", "ArrowUp", "Home", "End", "Enter", " "].includes(event.key)
    ) {
      return;
    }
    event.preventDefault();

    const focusable = focusItems();
    if (focusable.length === 0) return;

    const activeIndex = focusable.indexOf(
      document.activeElement as HTMLButtonElement,
    );

    switch (event.key) {
      case "ArrowDown":
        focusable[(activeIndex + 1) % focusable.length]?.focus();
        break;
      case "ArrowUp":
        focusable[
          (activeIndex - 1 + focusable.length) % focusable.length
        ]?.focus();
        break;
      case "Home":
        focusable[0]?.focus();
        break;
      case "End":
        focusable[focusable.length - 1]?.focus();
        break;
      case "Enter":
      case " ":
        // Activate the focused item (click triggers onClick which closes the menu).
        (document.activeElement as HTMLButtonElement | null)?.click();
        break;
    }
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

  const trigger = (
    <button
      type="button"
      className="menu-trigger"
      aria-haspopup="menu"
      aria-expanded={open}
      onClick={() => (open ? onClose() : onOpen())}
    >
      {label}
    </button>
  );

  /**
   * Compute fixed positioning from anchorRect when provided.
   * Falls back to null so the CSS absolute positioning is used when no anchor
   * is passed (e.g. during initial render or server-side).
   */
  const dropdownStyle = anchorRect
    ? {
        position: "fixed" as const,
        top: anchorRect.bottom + 1,
        left: anchorRect.left,
        minWidth: anchorRect.width,
      }
    : undefined;

  const dropdownContent = open ? (
    <div
      ref={portalRef}
      className="menu-dropdown"
      role="menu"
      aria-label={`${label} menu`}
      data-testid="menu-dropdown"
      style={dropdownStyle}
    >
      {renderChildren}
    </div>
  ) : null;

  /**
   * Render the dropdown via createPortal to document.body so it escapes the
   * menubar stacking context (--z-sticky: 200). Keyboard navigation (focus
   * trap, ArrowUp/Down, Home/End, Escape) and outside-click close are
   * preserved — both are wired via document-level event listeners.
   */
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
      {trigger}
      {open &&
        typeof document !== "undefined" &&
        createPortal(dropdownContent, document.body)}
    </div>
  );
}
