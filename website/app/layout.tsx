import type { Metadata } from "next";
import { Inter, JetBrains_Mono } from "next/font/google";

import { Providers } from "@/components/providers";

import "./globals.css";

const geist = Inter({
  subsets: ["latin"],
  variable: "--font-geist",
});

const mono = JetBrains_Mono({
  subsets: ["latin"],
  variable: "--font-mono",
});

export const metadata: Metadata = {
  metadataBase: new URL("https://ghostnote.app"),
  title: "GhostNote - Invisible AI Meeting Assistant",
  description:
    "The only AI meeting assistant that stays hidden during screen shares. 100% local, 100% private.",
  openGraph: {
    title: "GhostNote - Invisible AI Meeting Assistant",
    description:
      "The only AI meeting assistant that stays hidden during screen shares. 100% local, 100% private.",
    type: "website",
    url: "https://ghostnote.app",
  },
  twitter: {
    card: "summary_large_image",
    title: "GhostNote - Invisible AI Meeting Assistant",
    description:
      "The only AI meeting assistant that stays hidden during screen shares. 100% local, 100% private.",
  },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className={`${geist.variable} ${mono.variable}`}>
      <body className="min-h-screen bg-ink font-sans antialiased">
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify({
              "@context": "https://schema.org",
              "@type": "SoftwareApplication",
              name: "GhostNote",
              applicationCategory: "BusinessApplication",
              operatingSystem: "macOS, Windows, Linux",
              offers: {
                "@type": "Offer",
                price: "0",
                priceCurrency: "USD",
              },
              description:
                "Invisible AI meeting assistant that stays hidden during screen shares.",
            }),
          }}
        />
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
