import { useEffect, useState, type RefObject } from "react";

/** Types `text` out one character at a time. Instant when `instant` is true. */
export function useTypewriter(text: string, instant: boolean, speed = 24) {
  const [state, setState] = useState({ text, n: instant ? text.length : 0 });

  useEffect(() => {
    if (instant) {
      setState({ text, n: text.length });
      return;
    }
    setState({ text, n: 0 });
    let n = 0;
    const timer = window.setInterval(() => {
      n += 1;
      setState({ text, n });
      if (n >= text.length) window.clearInterval(timer);
    }, speed);
    return () => window.clearInterval(timer);
  }, [text, instant, speed]);

  // The state can lag one render behind a new `text`; never show the old count.
  const n = state.text === text ? state.n : 0;
  return { shown: text.slice(0, n), done: n >= text.length };
}

export function useMediaQuery(query: string) {
  const [matches, setMatches] = useState(false);
  useEffect(() => {
    const mq = window.matchMedia(query);
    const update = () => setMatches(mq.matches);
    update();
    mq.addEventListener("change", update);
    return () => mq.removeEventListener("change", update);
  }, [query]);
  return matches;
}

/** Pointer position relative to the element centre, each axis in -1..1. */
export function usePointerOffset(ref: RefObject<Element | null>, enabled: boolean) {
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  useEffect(() => {
    if (!enabled) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let frame = 0;
    const onMove = (event: PointerEvent) => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const el = ref.current;
        if (!el) return;
        const rect = el.getBoundingClientRect();
        const clamp = (v: number) => Math.max(-1, Math.min(1, v));
        setOffset({
          x: clamp((event.clientX - (rect.left + rect.width / 2)) / (window.innerWidth / 2)),
          y: clamp((event.clientY - (rect.top + rect.height / 2)) / (window.innerHeight / 2)),
        });
      });
    };
    window.addEventListener("pointermove", onMove);
    return () => {
      window.removeEventListener("pointermove", onMove);
      cancelAnimationFrame(frame);
    };
  }, [enabled, ref]);
  return offset;
}
