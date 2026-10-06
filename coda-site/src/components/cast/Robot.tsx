import { useRef } from "react";

import { usePointerOffset } from "../../lib/hooks";

const INK = "#16140f";
const W = 2.5;

interface Props {
  label: string;
  speaking?: boolean;
  happy?: boolean;
  track?: boolean;
  className?: string;
}

/** Patch: the contributor robot. Same ink language as the rest of the cast. */
export function Robot({ label, speaking = false, happy = false, track = false, className }: Props) {
  const ref = useRef<SVGSVGElement>(null);
  const gaze = usePointerOffset(ref, track);

  return (
    <svg
      ref={ref}
      viewBox="0 0 200 220"
      role="img"
      aria-label={label}
      className={`character h-full w-full overflow-visible ${speaking ? "is-speaking" : ""} ${className ?? ""}`}
    >
      <title>{label}</title>
      <ellipse className="shadow-pulse" cx="100" cy="210" rx="44" ry="5" fill={INK} opacity="0.12" />

      <g className="hover">
        {/* branch tail: a tiny git graph */}
        <g fill="none" stroke={INK} strokeWidth="2.2" strokeLinecap="round">
          <path d="M140 168 C160 168 166 152 178 146" />
          <path d="M160 160 C168 176 176 180 186 178" />
        </g>
        <circle cx="178" cy="146" r="5" fill="#5e7d62" stroke={INK} strokeWidth="2" />
        <circle cx="186" cy="178" r="5" fill="#d4521a" stroke={INK} strokeWidth="2" />

        {/* body */}
        <rect x="56" y="128" width="88" height="62" rx="20" fill="#e6d9c0" stroke={INK} strokeWidth={W} />
        <rect x="80" y="146" width="40" height="22" rx="7" fill="#fbf6ea" stroke={INK} strokeWidth="2" />
        <path d="M88 157 L95 162 L88 167" fill="none" stroke="#d4521a" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" />
        <path d="M100 166 H112" stroke={INK} strokeWidth="2.2" strokeLinecap="round" />

        {/* waving arm with wrench */}
        <g className={speaking ? "robot-wave" : "robot-idle"}>
          <path d="M58 146 C40 140 32 124 34 112" fill="none" stroke={INK} strokeWidth={W} strokeLinecap="round" />
          <circle cx="34" cy="110" r="5" fill="#e6d9c0" stroke={INK} strokeWidth="2" />
          <g transform="rotate(-20 34 100)">
            <rect x="30.5" y="78" width="7" height="28" rx="3" fill="#fbf6ea" stroke={INK} strokeWidth="2" />
            <path
              d="M24 70 C24 62 30 58 34 58 L34 68 L38 68 L38 58 C42 58 48 62 48 70 C48 77 42 81 34 81 C26 81 24 77 24 70 Z"
              fill="#fbf6ea"
              stroke={INK}
              strokeWidth="2"
              strokeLinejoin="round"
            />
          </g>
        </g>
        <path d="M142 146 C156 152 160 166 154 176" fill="none" stroke={INK} strokeWidth={W} strokeLinecap="round" />

        {/* neck */}
        <rect x="90" y="116" width="20" height="14" rx="4" fill="#cdbd9f" stroke={INK} strokeWidth="2" />

        {/* head with cat-ear antennas */}
        <g className={speaking ? "head-talk" : undefined}>
          <path d="M58 58 L52 26 L80 44 Z" fill="#e6d9c0" stroke={INK} strokeWidth={W} strokeLinejoin="round" />
          <path d="M142 58 L148 26 L120 44 Z" fill="#e6d9c0" stroke={INK} strokeWidth={W} strokeLinejoin="round" />
          <circle cx="52" cy="24" r="4" fill="#d4521a" stroke={INK} strokeWidth="1.8" />
          <circle cx="148" cy="24" r="4" fill="#d4521a" stroke={INK} strokeWidth="1.8" />
          <rect x="44" y="40" width="112" height="80" rx="30" fill="#f3ead8" stroke={INK} strokeWidth={W} />
          <rect x="58" y="54" width="84" height="52" rx="20" fill="#231c16" stroke={INK} strokeWidth="2" />

          <g
            style={{
              transform: `translate(${(gaze.x * 5).toFixed(2)}px, ${(gaze.y * 3).toFixed(2)}px)`,
              transition: "transform .35s cubic-bezier(.2,.8,.2,1)",
            }}
          >
            {happy ? (
              <g fill="none" stroke="#ffb27a" strokeWidth="3" strokeLinecap="round">
                <path d="M76 82 Q83 74 90 82" />
                <path d="M110 82 Q117 74 124 82" />
              </g>
            ) : (
              <g className="blink" style={{ animationDelay: "3.1s" }}>
                <rect x="78" y="70" width="10" height="16" rx="5" fill="#ffb27a" />
                <rect x="112" y="70" width="10" height="16" rx="5" fill="#ffb27a" />
              </g>
            )}
            {speaking ? (
              <ellipse className="talk" cx="100" cy="96" rx="6" ry="4" fill="#ffb27a" />
            ) : (
              <path d="M92 94 Q100 100 108 94" fill="none" stroke="#ffb27a" strokeWidth="2.6" strokeLinecap="round" />
            )}
          </g>
          <g fill="#d4521a" opacity="0.22">
            <circle cx="56" cy="96" r="5" />
            <circle cx="144" cy="96" r="5" />
          </g>
        </g>
      </g>
    </svg>
  );
}
