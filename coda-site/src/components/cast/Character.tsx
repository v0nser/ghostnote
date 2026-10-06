import { useId, useRef } from "react";

import type { Mood, Who } from "../../content/site";
import { usePointerOffset } from "../../lib/hooks";

const INK = "#16140f";
const MOUTH = "#3a1c16";
const W = 2.5;

type Gaze = { x: number; y: number };

interface Props {
  who: Who;
  label: string;
  mood?: Mood;
  speaking?: boolean;
  dim?: boolean;
  /** Where the pupils point, -1..1 per axis. Ignored when `track` is on. */
  look?: Gaze;
  /** Follow the visitor's pointer. */
  track?: boolean;
  /** Crop to a film close-up. */
  close?: boolean;
  className?: string;
}

export function Character({
  who,
  label,
  mood = "calm",
  speaking = false,
  dim = false,
  look,
  track = false,
  close = false,
  className,
}: Props) {
  const ref = useRef<SVGSVGElement>(null);
  const pointer = usePointerOffset(ref, track);
  const gaze = track ? pointer : (look ?? { x: 0, y: 0 });
  const blinkDelay = who === "engineer" ? "0.4s" : who === "friend" ? "2.1s" : "1.2s";

  return (
    <svg
      ref={ref}
      viewBox={close ? "48 28 104 132" : "0 0 200 200"}
      role="img"
      aria-label={label}
      className={`character h-full w-full overflow-visible ${speaking ? "is-speaking is-hot" : ""} ${dim ? "is-dim" : ""} ${className ?? ""}`}
    >
      <title>{label}</title>
      {who === "coda" ? (
        <Coda mood={mood} speaking={speaking} gaze={gaze} blinkDelay={blinkDelay} />
      ) : (
        <Human who={who} mood={mood} speaking={speaking} gaze={gaze} blinkDelay={blinkDelay} />
      )}
    </svg>
  );
}

interface PartProps {
  mood: Mood;
  speaking: boolean;
  gaze: Gaze;
  blinkDelay: string;
}

function Human({ who, mood, speaking, gaze, blinkDelay }: PartProps & { who: Who }) {
  const engineer = who === "engineer";
  const skin = engineer ? "#ecc9a4" : "#c99670";
  const hair = engineer ? "#25201b" : "#4b2f22";
  const top = engineer ? "#2c2b28" : "#7a9a7e";

  return (
    <g className="breathe">
      <path
        d="M24 204 C28 154 60 134 100 134 C140 134 172 154 176 204 Z"
        fill={top}
        stroke={INK}
        strokeWidth={W}
        strokeLinejoin="round"
      />
      {engineer ? (
        <>
          <path d="M68 140 C80 162 120 162 132 140" fill="none" stroke="#4d4a44" strokeWidth={W} strokeLinecap="round" />
          <path d="M91 154 L89 180 M109 154 L111 180" stroke="#d4521a" strokeWidth={W} strokeLinecap="round" />
          <path
            className={speaking ? "arm-talk" : undefined}
            d="M48 148 C28 138 22 118 34 108"
            fill="none"
            stroke={INK}
            strokeWidth={W}
            strokeLinecap="round"
          />
        </>
      ) : (
        <path d="M78 137 C86 152 114 152 122 137" fill="none" stroke={INK} strokeWidth={W} strokeLinecap="round" />
      )}
      <rect x="88" y="114" width="24" height="26" rx="9" fill={skin} stroke={INK} strokeWidth={W} />

      <g className={speaking ? "head head-talk" : "head"}>
        {!engineer && <circle cx="100" cy="40" r="15" fill={hair} stroke={INK} strokeWidth={W} />}
        <ellipse cx="61" cy="94" rx="7" ry="9" fill={skin} stroke={INK} strokeWidth={W} />
        <ellipse cx="139" cy="94" rx="7" ry="9" fill={skin} stroke={INK} strokeWidth={W} />
        <ellipse cx="100" cy="88" rx="40" ry="43" fill={skin} stroke={INK} strokeWidth={W} />
        {engineer ? (
          <>
            <path
              d="M60 84 C56 50 80 38 102 40 C126 40 146 56 140 86 C134 70 124 62 112 66 C108 56 94 56 90 64 C82 58 68 64 62 80 Z"
              fill={hair}
              stroke={INK}
              strokeWidth={W}
              strokeLinejoin="round"
            />
            <path d="M96 42 C92 30 104 24 112 30 M118 44 C124 34 134 36 136 44" fill="none" stroke={INK} strokeWidth={W} strokeLinecap="round" />
          </>
        ) : (
          <path
            d="M58 96 C52 60 74 46 100 46 C126 46 148 60 142 96 C138 80 130 70 118 66 C104 72 84 72 70 70 C64 78 60 86 58 96 Z"
            fill={hair}
            stroke={INK}
            strokeWidth={W}
            strokeLinejoin="round"
          />
        )}
        <Brows mood={mood} />
        <Eyes mood={mood} gaze={gaze} blinkDelay={blinkDelay} />
        {!engineer && (
          <g fill="rgba(255,255,255,0.14)" stroke={INK} strokeWidth="2.2">
            <circle cx="84" cy="93" r="12" />
            <circle cx="116" cy="93" r="12" />
            <path d="M96 92 Q100 88 104 92" fill="none" />
          </g>
        )}
        {(mood === "happy" || mood === "calm" || mood === "asleep") && (
          <g fill="#d4521a" opacity="0.16">
            <circle cx="72" cy="107" r="6" />
            <circle cx="128" cy="107" r="6" />
          </g>
        )}
        <HumanMouth mood={mood} speaking={speaking} />
      </g>
      {mood === "asleep" && <Zzz />}
    </g>
  );
}

const BROWS: Record<Mood, [string, string]> = {
  tired: ["M75 83 Q83 80 92 78", "M108 78 Q117 80 125 83"],
  confused: ["M75 79 Q83 70 92 77", "M108 81 L125 82"],
  curious: ["M75 78 Q83 71 92 76", "M108 76 Q117 71 125 78"],
  calm: ["M76 80 Q84 76 92 79", "M108 79 Q116 76 124 80"],
  happy: ["M76 79 Q84 74 92 78", "M108 78 Q116 74 124 79"],
  asleep: ["M76 84 Q84 82 92 84", "M108 84 Q116 82 124 84"],
};

function Brows({ mood }: { mood: Mood }) {
  const [left, right] = BROWS[mood];
  return (
    <g fill="none" stroke={INK} strokeWidth="2.6" strokeLinecap="round" className="brows">
      <path d={left} />
      <path d={right} />
    </g>
  );
}

function Eyes({ mood, gaze, blinkDelay }: { mood: Mood; gaze: Gaze; blinkDelay: string }) {
  if (mood === "asleep") {
    return (
      <g fill="none" stroke={INK} strokeWidth={W} strokeLinecap="round">
        <path d="M77 94 Q84 99 91 94" />
        <path d="M109 94 Q116 99 123 94" />
      </g>
    );
  }
  const ry = mood === "tired" ? 3.4 : mood === "curious" ? 6.2 : 5.2;
  return (
    <>
      <g style={{ transform: `translate(${(gaze.x * 3.4).toFixed(2)}px, ${(gaze.y * 2.4).toFixed(2)}px)`, transition: "transform .35s cubic-bezier(.2,.8,.2,1)" }}>
        <g className="blink" style={{ animationDelay: blinkDelay }}>
          <ellipse cx="84" cy="93" rx="4.4" ry={ry} fill={INK} />
          <ellipse cx="116" cy="93" rx="4.4" ry={ry} fill={INK} />
          <circle cx="85.6" cy="91" r="1.3" fill="#fff" />
          <circle cx="117.6" cy="91" r="1.3" fill="#fff" />
        </g>
      </g>
      {mood === "tired" && (
        <g fill="none" stroke={INK} strokeWidth="2" strokeLinecap="round" opacity="0.45">
          <path d="M78 102 Q84 105 90 102" />
          <path d="M110 102 Q116 105 122 102" />
        </g>
      )}
    </>
  );
}

function HumanMouth({ mood, speaking }: { mood: Mood; speaking: boolean }) {
  if (speaking) return <ellipse className="talk" cx="100" cy="115" rx="6.5" ry="5.5" fill={MOUTH} />;
  const line = { fill: "none", stroke: INK, strokeWidth: W, strokeLinecap: "round" as const };
  switch (mood) {
    case "happy":
      return <path d="M89 111 Q100 124 111 111 Z" fill={MOUTH} stroke={INK} strokeWidth="2" strokeLinejoin="round" />;
    case "tired":
      return <path d="M93 116 Q100 113 107 116" {...line} />;
    case "confused":
      return <path d="M92 115 Q96 111 100 115 Q104 119 108 114" {...line} />;
    case "curious":
      return <circle cx="100" cy="115" r="3.4" fill={MOUTH} />;
    case "asleep":
      return <path d="M96 114 Q100 117 104 114" {...line} />;
    default:
      return <path d="M92 113 Q100 119 108 113" {...line} />;
  }
}

function Zzz() {
  return (
    <g className="zzz" fill={INK} fontFamily="JetBrains Mono, ui-monospace, monospace" fontWeight="600">
      <text x="148" y="52" fontSize="16">z</text>
      <text x="162" y="36" fontSize="12">z</text>
    </g>
  );
}

function Coda({ mood, speaking, gaze, blinkDelay }: PartProps) {
  const raw = useId();
  const id = `coda${raw.replace(/[^a-zA-Z0-9]/g, "")}`;
  return (
    <>
      <defs>
        <radialGradient id={id} cx="36%" cy="30%" r="78%">
          <stop offset="0%" stopColor="#ffd8b2" />
          <stop offset="42%" stopColor="#ff8c4f" />
          <stop offset="100%" stopColor="#e4501b" />
        </radialGradient>
      </defs>
      <ellipse className="shadow-pulse" cx="100" cy="186" rx="34" ry="5" fill={INK} opacity="0.12" />
      <g className="hover">
        <ellipse
          className="orbit"
          cx="100"
          cy="106"
          rx="76"
          ry="20"
          fill="none"
          stroke={INK}
          strokeOpacity="0.3"
          strokeWidth="1.6"
          strokeDasharray="2 7"
          strokeLinecap="round"
          transform="rotate(-10 100 106)"
        />
        <path d="M100 57 C97 44 104 36 115 36 C113 46 108 54 100 57 Z" fill="#7a9a7e" stroke={INK} strokeWidth="2.2" strokeLinejoin="round" />
        <circle cx="100" cy="104" r="46" fill={`url(#${id})`} stroke={INK} strokeWidth={W} />
        <ellipse cx="82" cy="82" rx="13" ry="8" fill="#fff" opacity="0.45" transform="rotate(-24 82 82)" />
        <g style={{ transform: `translate(${(gaze.x * 6).toFixed(2)}px, ${(gaze.y * 4).toFixed(2)}px)`, transition: "transform .35s cubic-bezier(.2,.8,.2,1)" }}>
          {mood === "asleep" ? (
            <g fill="none" stroke={INK} strokeWidth={W} strokeLinecap="round">
              <path d="M80 100 Q86 105 92 100" />
              <path d="M108 100 Q114 105 120 100" />
            </g>
          ) : (
            <g className="blink" style={{ animationDelay: blinkDelay }}>
              <rect x="81" y="90" width="10" height="18" rx="5" fill={INK} />
              <rect x="109" y="90" width="10" height="18" rx="5" fill={INK} />
              <circle cx="88" cy="94" r="1.6" fill="#fff" />
              <circle cx="116" cy="94" r="1.6" fill="#fff" />
            </g>
          )}
          {speaking ? (
            <ellipse className="talk" cx="100" cy="121" rx="5.5" ry="4.5" fill={MOUTH} />
          ) : mood === "happy" ? (
            <path d="M90 117 Q100 128 110 117 Z" fill={MOUTH} stroke={INK} strokeWidth="2" strokeLinejoin="round" />
          ) : (
            <path d="M92 119 Q100 124 108 119" fill="none" stroke={INK} strokeWidth={W} strokeLinecap="round" />
          )}
        </g>
      </g>
    </>
  );
}
