// Inline stroke icons (24px grid), decorative unless labelled by their button.
import type { ReactNode } from "react";

function Icon({ size = 18, strokeWidth = 2, children }: { size?: number; strokeWidth?: number; children: ReactNode }) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      {children}
    </svg>
  );
}

type P = { size?: number };

export const LogoMark = () => (
  <Icon size={18} strokeWidth={2.2}>
    <path d="M4 7h11" stroke="#fff" />
    <path d="M4 12h8" stroke="#fff" />
    <path d="M4 17h5" stroke="#fff" />
    <path d="M16 11l5 3.5-5 3.5z" stroke="#fff" fill="#fff" />
  </Icon>
);
export const ListIcon = ({ size }: P) => (
  <Icon size={size}>
    <path d="M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01" />
  </Icon>
);
export const CheckCircleIcon = ({ size }: P) => (
  <Icon size={size}>
    <circle cx="12" cy="12" r="9" />
    <path d="M8 12.5l2.5 2.5L16 9.5" />
  </Icon>
);
export const AlertIcon = ({ size }: P) => (
  <Icon size={size}>
    <path d="M12 3l9.5 17h-19z" />
    <path d="M12 10v4M12 17h.01" />
  </Icon>
);
export const GearIcon = ({ size }: P) => (
  <Icon size={size}>
    <circle cx="12" cy="12" r="3" />
    <path d="M12 2v3M12 19v3M4.9 4.9 7 7M17 17l2.1 2.1M2 12h3M19 12h3M4.9 19.1 7 17M17 7l2.1-2.1" />
  </Icon>
);
export const LinkIcon = ({ size }: P) => (
  <Icon size={size}>
    <path d="M10 13a5 5 0 0 0 7.1 0l3-3a5 5 0 0 0-7.1-7.1l-1.1 1.1" />
    <path d="M14 11a5 5 0 0 0-7.1 0l-3 3a5 5 0 0 0 7.1 7.1l1.1-1.1" />
  </Icon>
);
export const PlusIcon = ({ size = 16 }: P) => (
  <Icon size={size} strokeWidth={2.4}>
    <path d="M12 5v14M5 12h14" />
  </Icon>
);
export const CloseIcon = ({ size = 16 }: P) => (
  <Icon size={size} strokeWidth={2.2}>
    <path d="M6 6l12 12M18 6 6 18" />
  </Icon>
);
export const RetryIcon = ({ size = 16 }: P) => (
  <Icon size={size}>
    <path d="M3 12a9 9 0 1 0 3-6.7" />
    <path d="M3 4v5h5" />
  </Icon>
);
export const PauseIcon = ({ size = 16 }: P) => (
  <Icon size={size}>
    <path d="M9 5v14M15 5v14" />
  </Icon>
);
export const PlayIcon = ({ size = 16 }: P) => (
  <Icon size={size}>
    <path d="M7 5l12 7-12 7z" />
  </Icon>
);
export const FolderIcon = ({ size = 16 }: P) => (
  <Icon size={size}>
    <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
  </Icon>
);
export const ClipboardIcon = ({ size = 16 }: P) => (
  <Icon size={size}>
    <rect x="8" y="3" width="8" height="4" rx="1" />
    <path d="M16 5h2a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V7a2 2 0 0 1 2-2h2" />
  </Icon>
);
export const VideoIcon = ({ size = 22 }: P) => (
  <Icon size={size}>
    <rect x="3" y="5" width="18" height="14" rx="2" />
    <path d="M10 9l5 3-5 3z" />
  </Icon>
);
export const MusicIcon = ({ size = 22 }: P) => (
  <Icon size={size}>
    <path d="M9 18V5l12-2v13" />
    <circle cx="6" cy="18" r="3" />
    <circle cx="18" cy="16" r="3" />
  </Icon>
);
export const PlaylistIcon = ({ size = 22 }: P) => (
  <Icon size={size}>
    <path d="M3 6h12M3 12h12M3 18h8M17 14l5 3-5 3z" />
  </Icon>
);
