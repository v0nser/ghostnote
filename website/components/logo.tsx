import { cn } from "@/lib/utils";

export function Logo({
  className,
  showWordmark = true,
  size = 36,
}: {
  className?: string;
  showWordmark?: boolean;
  size?: number;
}) {
  return (
    <a href="/#top" className={cn("group flex items-center gap-2.5 focus-ring rounded-full", className)}>
      <svg width={size} height={size} viewBox="0 0 64 64" aria-hidden>
        <defs>
          <linearGradient id="ghostFill" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#e8e8ea" />
            <stop offset="100%" stopColor="#6e6e74" />
          </linearGradient>
        </defs>
        <path
          d="M32 6c12 0 20 9 20 20v24c0 3-3.2 4.4-5.4 2.4L40 46l-5.6 5.4c-1.3 1.2-3.5 1.2-4.8 0L24 46l-6.6 6.4C15.2 54.4 12 53 12 50V26C12 15 20 6 32 6z"
          fill="url(#ghostFill)"
        />
        <rect x="24" y="22" width="16" height="18" rx="2.5" fill="#050505" opacity="0.88" />
        <path d="M27 27h10M27 31.5h10M27 36h7" stroke="#c8c8cc" strokeWidth="1.6" strokeLinecap="round" />
        <circle cx="24" cy="20" r="2.1" fill="#050505" />
        <circle cx="40" cy="20" r="2.1" fill="#050505" />
      </svg>
      {showWordmark ? (
        <span className="text-sm font-semibold tracking-[0.28em] text-white">GHOSTNOTE</span>
      ) : (
        <span className="sr-only">GhostNote</span>
      )}
    </a>
  );
}
