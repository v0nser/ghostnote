"use client";

import { type MouseEvent, type ReactNode, useRef, useState } from "react";

import { cn } from "@/lib/utils";

export function MagneticButton({
  children,
  href,
  onClick,
  type = "button",
  variant = "primary",
  className,
  download,
}: {
  children: ReactNode;
  href?: string;
  onClick?: () => void;
  type?: "button" | "submit";
  variant?: "primary" | "secondary" | "ghost";
  className?: string;
  download?: boolean | string;
}) {
  const ref = useRef<HTMLAnchorElement | HTMLButtonElement>(null);
  const [ripples, setRipples] = useState<{ id: number; x: number; y: number }[]>([]);

  const burst = (event: MouseEvent<HTMLElement>) => {
    const node = ref.current;
    if (!node) return;
    const box = node.getBoundingClientRect();
    const id = Date.now();
    setRipples((prev) => [
      ...prev,
      { id, x: event.clientX - box.left, y: event.clientY - box.top },
    ]);
    window.setTimeout(() => {
      setRipples((prev) => prev.filter((ripple) => ripple.id !== id));
    }, 600);
    onClick?.();
  };

  const styles = {
    primary: "bg-[#c8c8cc] text-ink hover:bg-white",
    secondary: "border border-white/15 bg-white/5 text-white hover:border-white/30 hover:bg-white/10",
    ghost: "text-mist hover:text-white",
  }[variant];

  const shared = cn(
    "focus-ring relative inline-flex min-h-11 items-center justify-center overflow-hidden rounded-full px-6 py-2.5 text-sm font-semibold transition-colors",
    styles,
    className,
  );

  const inner = (
    <>
      <span className="relative z-10">{children}</span>
      {ripples.map((ripple) => (
        <span
          key={ripple.id}
          className="pointer-events-none absolute h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white/40"
          style={{ left: ripple.x, top: ripple.y, animation: "ripple 0.6s ease-out forwards" }}
        />
      ))}
    </>
  );

  if (href) {
    const external = href.startsWith("http");
    return (
      <a
        ref={ref as React.RefObject<HTMLAnchorElement>}
        href={href}
        className={shared}
        onClick={burst}
        {...(download ? { download: download === true ? true : download } : {})}
        {...(external && !download ? { target: "_blank", rel: "noreferrer" } : {})}
      >
        {inner}
      </a>
    );
  }

  return (
    <button
      ref={ref as React.RefObject<HTMLButtonElement>}
      type={type}
      className={shared}
      onClick={burst}
    >
      {inner}
    </button>
  );
}
