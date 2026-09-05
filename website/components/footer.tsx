import { Linkedin, MessageCircle, Twitter } from "lucide-react";

import { GitHubBadge } from "@/components/github-badge";
import { Logo } from "@/components/logo";
import { repoUrl } from "@/lib/repo";

const COLUMNS = [
  {
    title: "Product",
    links: [
      ["Features", "/#features"],
      ["Early Bird", "/#early-bird"],
      ["Pricing", "/#pricing"],
      ["Download", "/#cta"],
    ],
  },
  {
    title: "Open Source",
    links: [
      ["Contribute", "/#contribute"],
      ["GitHub", repoUrl()],
      ["Issues", repoUrl("/issues")],
      ["New issue", repoUrl("/issues/new")],
      ["Account", "/account"],
    ],
  },
  {
    title: "Resources",
    links: [
      ["Docs", "/#how"],
      ["Community", "/#contribute"],
      ["Support", "/#faq"],
      ["Checkout", "/checkout?plan=pro"],
    ],
  },
  {
    title: "Legal",
    links: [
      ["Privacy", "/#faq"],
      ["Terms", "/#faq"],
      ["Security", "/#faq"],
    ],
  },
];

export function Footer() {
  return (
    <footer className="border-t border-white/8 px-5 py-14">
      <div className="mx-auto grid max-w-6xl gap-10 md:grid-cols-[1.2fr_2fr]">
        <div>
          <Logo />
          <p className="mt-4 max-w-xs text-sm text-mist">
            Invisible intelligence. Unforgettable meetings.
          </p>
        </div>
        <div className="grid grid-cols-2 gap-8 sm:grid-cols-4">
          {COLUMNS.map((column) => (
            <div key={column.title}>
              <p className="text-xs uppercase tracking-[0.18em] text-white/50">{column.title}</p>
              <ul className="mt-3 space-y-2">
                {column.links.map(([label, href]) => (
                  <li key={label}>
                    <a href={href} className="link-underline text-sm text-mist hover:text-white">
                      {label}
                    </a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </div>
      <div className="mx-auto mt-10 flex max-w-6xl flex-wrap items-center justify-between gap-4 border-t border-white/8 pt-6">
        <GitHubBadge />
        <div className="flex gap-2">
          {[Twitter, MessageCircle, Linkedin].map((Icon, index) => (
            <a
              key={index}
              href="/#contribute"
              aria-label="Social"
              className="focus-ring flex size-9 items-center justify-center rounded-full border border-white/10 text-mist transition hover:scale-110 hover:border-cyan/40 hover:text-white"
            >
              <Icon className="size-4" />
            </a>
          ))}
        </div>
      </div>
    </footer>
  );
}
