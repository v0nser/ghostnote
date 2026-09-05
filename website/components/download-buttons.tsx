"use client";

import { useEffect, useState } from "react";
import { Apple, Monitor } from "lucide-react";

import { MagneticButton } from "@/components/magnetic-button";
import { MAC_DOWNLOAD, WIN_DOWNLOAD } from "@/lib/downloads";

type Manifest = {
  mac?: { available: boolean; url?: string };
  windows?: { available: boolean; url?: string };
};

export function DownloadButtons({
  align = "center",
}: {
  align?: "center" | "start";
}) {
  const [manifest, setManifest] = useState<Manifest | null>(null);

  useEffect(() => {
    fetch("/downloads/latest.json", { cache: "no-store" })
      .then((res) => (res.ok ? res.json() : null))
      .then(setManifest)
      .catch(() => undefined);
  }, []);

  const macReady = manifest?.mac?.available !== false;
  const winReady = manifest?.windows?.available === true;

  return (
    <div>
      <div className={align === "center" ? "flex flex-wrap justify-center gap-3" : "flex flex-wrap gap-3"}>
        <MagneticButton href={manifest?.mac?.url ?? MAC_DOWNLOAD} download="GhostNote.dmg">
          <span className="inline-flex items-center gap-2">
            <Apple className="size-4" /> Download for Mac
          </span>
        </MagneticButton>
        {winReady ? (
          <MagneticButton href={manifest?.windows?.url ?? WIN_DOWNLOAD} download="GhostNote-Setup.exe" variant="secondary">
            <span className="inline-flex items-center gap-2">
              <Monitor className="size-4" /> Download for Windows
            </span>
          </MagneticButton>
        ) : (
          <MagneticButton href="#cta" variant="secondary">
            <span className="inline-flex items-center gap-2">
              <Monitor className="size-4" /> Windows .exe (build on Windows)
            </span>
          </MagneticButton>
        )}
      </div>
      {macReady && !winReady ? (
        <p className={`mt-3 text-xs text-mist ${align === "center" ? "text-center" : ""}`}>
          The Mac .dmg is ready. The Windows .exe has to be built on a Windows machine with{" "}
          <code className="font-mono">npm run tauri:build</code>, then{" "}
          <code className="font-mono">npm run installers:publish</code>.
        </p>
      ) : null}
    </div>
  );
}
