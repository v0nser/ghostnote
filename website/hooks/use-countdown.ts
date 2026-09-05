"use client";

import { useEffect, useState } from "react";

import { OFFER_STORAGE_KEY, OFFER_WINDOW_DAYS } from "@/lib/plans";

export type TimeLeft = {
  days: number;
  hours: number;
  minutes: number;
  seconds: number;
  expired: boolean;
  target: number;
};

function padParts(ms: number): TimeLeft {
  if (ms <= 0) {
    return { days: 0, hours: 0, minutes: 0, seconds: 0, expired: true, target: 0 };
  }
  return {
    days: Math.floor(ms / (1000 * 60 * 60 * 24)),
    hours: Math.floor((ms / (1000 * 60 * 60)) % 24),
    minutes: Math.floor((ms / 1000 / 60) % 60),
    seconds: Math.floor((ms / 1000) % 60),
    expired: false,
    target: 0,
  };
}

export function resolveOfferEnd(now = Date.now()) {
  if (typeof window === "undefined") {
    return now + OFFER_WINDOW_DAYS * 24 * 60 * 60 * 1000;
  }
  const stored = window.localStorage.getItem(OFFER_STORAGE_KEY);
  if (stored) {
    const parsed = Number(stored);
    if (!Number.isNaN(parsed)) return parsed;
  }
  const target = now + OFFER_WINDOW_DAYS * 24 * 60 * 60 * 1000;
  window.localStorage.setItem(OFFER_STORAGE_KEY, String(target));
  return target;
}

export function useCountdown() {
  const [time, setTime] = useState<TimeLeft>({
    days: OFFER_WINDOW_DAYS,
    hours: 0,
    minutes: 0,
    seconds: 0,
    expired: false,
    target: 0,
  });

  useEffect(() => {
    const target = resolveOfferEnd();
    const tick = () => {
      const next = padParts(target - Date.now());
      setTime({ ...next, target });
    };
    tick();
    const id = window.setInterval(tick, 1000);
    return () => window.clearInterval(id);
  }, []);

  return time;
}
