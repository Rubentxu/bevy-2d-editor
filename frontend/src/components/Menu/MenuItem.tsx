import type { MenuItem as MenuItemData } from "../../data/menus";

export default function MenuItem({
  label,
  shortcut,
  disabled,
  onClick,
  submenu,
  testId,
}: MenuItemData) {
  return (
    <button
      type="button"
      className="menu-item"
      role="menuitem"
      disabled={disabled}
      onClick={onClick}
      data-testid={testId}
      title={shortcut ? `${label} (${shortcut})` : label}
    >
      <span>{label}</span>
      <span className="menu-item-shortcut">{submenu ? "›" : shortcut}</span>
    </button>
  );
}
