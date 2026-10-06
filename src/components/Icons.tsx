type icon_props = { size?: number };

function svg(path: string, size = 18) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path d={path} stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

export function IconFiles({ size }: icon_props) {
  return svg("M4 5.5h6l2 2H20v11H4z", size);
}

export function IconSearch({ size }: icon_props) {
  return svg("M11 18a7 7 0 1 1 0-14 7 7 0 0 1 0 14zM20 20l-3.5-3.5", size);
}

export function IconBookmark({ size }: icon_props) {
  return svg("M7 4h10v16l-5-3-5 3z", size);
}

export function IconGraph({ size }: icon_props) {
  return (
    <svg width={size ?? 18} height={size ?? 18} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <circle cx="6" cy="12" r="2.2" stroke="currentColor" strokeWidth="1.7" />
      <circle cx="17" cy="6" r="2.2" stroke="currentColor" strokeWidth="1.7" />
      <circle cx="17" cy="17" r="2.2" stroke="currentColor" strokeWidth="1.7" />
      <path d="M8 11.2 15 7.2M8 13.2l7 3.2" stroke="currentColor" strokeWidth="1.7" />
    </svg>
  );
}

export function IconDaily({ size }: icon_props) {
  return svg("M5 5h14v14H5zM5 9h14M9 3v4M15 3v4", size);
}

export function IconSettings({ size }: icon_props) {
  return svg("M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7zM12 3.5v2.2M12 18.3v2.2M4.8 7.2l1.9 1.1M17.3 15.7l1.9 1.1M4.8 16.8l1.9-1.1M17.3 8.3l1.9-1.1", size);
}

export function IconClose({ size }: icon_props) {
  return svg("M6 6l12 12M18 6 6 18", size);
}

export function IconPlus({ size }: icon_props) {
  return svg("M12 5v14M5 12h14", size);
}
