import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./app/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        ink: "#000000",
        charcoal: "#161618",
        cyan: "#c8c8cc",
        violet: "#6e6e74",
        neon: "#9a9aa0",
        mist: "#8a8a90",
        accent: {
          cyan: "#22d3ee",
          purple: "#a78bfa",
          blue: "#38bdf8",
        },
      },
      animation: {
        "float-slow": "floatSlow 9s ease-in-out infinite",
        "urgent-shake": "urgentShake 0.8s ease-in-out infinite",
        "subtle-shake": "subtleShake 2.8s ease-in-out infinite",
      },
      keyframes: {
        floatSlow: {
          "0%, 100%": { transform: "translateY(0px)" },
          "50%": { transform: "translateY(-10px)" },
        },
        urgentShake: {
          "0%, 100%": { transform: "translateX(0)" },
          "30%": { transform: "translateX(-2px)" },
          "60%": { transform: "translateX(2px)" },
        },
        subtleShake: {
          "0%, 100%": { transform: "rotate(0deg)" },
          "50%": { transform: "rotate(-0.6deg)" },
        },
      },
      fontFamily: {
        sans: ["var(--font-geist)", "ui-sans-serif", "system-ui"],
        mono: ["var(--font-mono)", "ui-monospace", "SFMono-Regular"],
      },
      boxShadow: {
        glow: "0 0 24px rgba(200, 200, 204, 0.08)",
        "glow-lg": "0 0 40px rgba(200, 200, 204, 0.12)",
      },
      backgroundImage: {
        "cyan-purple": "linear-gradient(135deg, #e8e8ea 0%, #9a9aa0 100%)",
        "blue-cyan": "linear-gradient(135deg, #6e6e74 0%, #c8c8cc 100%)",
      },
    },
  },
  plugins: [],
};

export default config;
