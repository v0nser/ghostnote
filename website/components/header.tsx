"use client";

import { useState } from "react";
import { Github, Menu, X } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";

import { EarlyBirdBanner } from "@/components/early-bird-banner";
import { Logo } from "@/components/logo";
import { MagneticButton } from "@/components/magnetic-button";

const LINKS = [
  { href: "/#demo", label: "Demo" },
  { href: "/#features", label: "Features" },
  { href: "/#how", label: "How it works" },
  { href: "/#pricing", label: "Pricing", badge: "Early Bird" },
  { href: "/#contribute", label: "Contribute", icon: true },
  { href: "/#faq", label: "FAQ" },
];

export function Header() {
  const [open, setOpen] = useState(false);

  return (
    <div className="sticky top-0 z-50">
      <EarlyBirdBanner />
      <header className="border-b border-white/5 bg-ink/80 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-5">
        <Logo />
        <nav className="hidden items-center gap-7 md:flex" aria-label="Primary">
          {LINKS.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="focus-ring inline-flex items-center gap-1.5 text-sm text-mist transition-colors hover:text-white"
            >
              {"icon" in link && link.icon ? <Github className="size-3.5" /> : null}
              {link.label}
              {"badge" in link && link.badge ? (
                <span className="relative ml-0.5 inline-flex size-2">
                  <span className="absolute inline-flex size-full animate-ping rounded-full bg-rose-400 opacity-75" />
                  <span className="relative inline-flex size-2 rounded-full bg-rose-400" />
                  <span className="sr-only">{link.badge}</span>
                </span>
              ) : null}
            </a>
          ))}
        </nav>
        <div className="hidden md:block">
          <MagneticButton href="/#early-bird">Claim Early Bird</MagneticButton>
        </div>
        <button
          type="button"
          className="focus-ring rounded-full p-2 text-white md:hidden"
          aria-label={open ? "Close menu" : "Open menu"}
          aria-expanded={open}
          onClick={() => setOpen((value) => !value)}
        >
          {open ? <X className="size-5" /> : <Menu className="size-5" />}
        </button>
      </div>
      <AnimatePresence>
        {open ? (
          <motion.nav
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: "auto", opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="overflow-hidden border-t border-white/5 md:hidden"
            aria-label="Mobile"
          >
            <div className="flex flex-col gap-3 px-5 py-4">
              {LINKS.map((link) => (
                <a
                  key={link.href}
                  href={link.href}
                  className="focus-ring inline-flex items-center gap-2 py-2 text-sm text-mist"
                  onClick={() => setOpen(false)}
                >
                  {link.label}
                  {"badge" in link && link.badge ? (
                    <span className="size-2 rounded-full bg-rose-400" aria-hidden />
                  ) : null}
                </a>
              ))}
              <MagneticButton href="/#early-bird" className="w-full">
                Claim Early Bird
              </MagneticButton>
            </div>
          </motion.nav>
        ) : null}
      </AnimatePresence>
    </header>
    </div>
  );
}
