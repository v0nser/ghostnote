import type { ReactNode } from "react";
import { CheckCircle2, Circle, Download, ExternalLink } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { useAgentStore } from "@/store/agent";

export function SetupPanel() {
  const { setup, download, error, downloadWhisper, setAutostart, setView } = useAgentStore();
  const windows = setup?.platform === "windows";
  const percent =
    download && download.total > 0 ? Math.round((download.received / download.total) * 100) : null;

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-y-auto px-6 py-6">
      <h1 className="text-lg font-medium">Set up Coda</h1>
      <p className="mt-1 max-w-xl text-sm leading-relaxed text-muted-foreground">
        Everything stays on this machine. Two local pieces: a speech model and
        Ollama. {windows ? "Windows does not need a virtual audio driver." : null}
      </p>

      <ol className="mt-6 max-w-xl space-y-4">
        <Step
          done={Boolean(setup?.whisperInstalled)}
          title="Speech model (small.en)"
          body={
            setup?.whisperInstalled
              ? `Installed ${setup.whisperModel}`
              : "Download ggml-small.en (~466 MB). Much better on accents than base.en. Stays on this machine."
          }
        >
          {!setup?.whisperInstalled ? (
            <Button type="button" size="sm" onClick={() => void downloadWhisper()}>
              <Download className="size-3.5" />
              {percent !== null ? `Downloading ${percent}%` : "Download model"}
            </Button>
          ) : null}
        </Step>

        <Step
          done={Boolean(setup?.ollamaAvailable)}
          title="Ollama"
          body={
            setup?.ollamaAvailable
              ? `Ready · ${setup.ollamaModel ?? "local model"}`
              : "Install Ollama, then pull a small model so answers start quickly."
          }
        >
          {!setup?.ollamaAvailable ? (
            <a
              href="https://ollama.com/download"
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1.5 text-xs text-stealth hover:underline"
            >
              Open ollama.com/download <ExternalLink className="size-3" />
            </a>
          ) : null}
        </Step>

        <Step
          done={Boolean(setup?.ollamaModel)}
          title="A fast local model"
          body="Live meeting answers use a small model. The agent needs a larger one to summarize PDFs and follow tools."
        >
          <div className="flex flex-col gap-1.5">
            <code className="rounded-md border border-white/10 bg-black/30 px-2 py-1 font-mono text-[11px]">
              ollama pull llama3.2:3b
            </code>
            <code className="rounded-md border border-white/10 bg-black/30 px-2 py-1 font-mono text-[11px]">
              ollama pull llama3.1:8b
            </code>
          </div>
        </Step>

        <Step
          done
          title="Models"
          body="Speech uses Whisper on this machine. Simple commands skip the LLM. Agent work prefers llama3.1:8b when it is installed."
        >
          <p className="text-[11px] text-muted-foreground">
            Fast router: deterministic. Agent: {setup?.ollamaModel ?? "Ollama default"}. Speech:{" "}
            {setup?.whisperModel ?? "not installed"}.
          </p>
        </Step>

        <Step
          done
          title="Permissions"
          body="I ask before email, calendar, or delete. After you approve, Mail/Outlook sends, Calendar creates the event, and Reminders sets the alarm. macOS will prompt for Automation access — allow Coda. Because Coda is a new app identity, re-grant Microphone, Screen Recording, Notifications, and Automation. Files stay in Downloads, Documents, Desktop, and Home. PDFs are read locally. Existing Ghost Note data is copied into ~/.coda and left in place."
        />

        <Step
          done
          title="Screen ask"
          body="⌘⇧C (Ctrl+Shift+C on Windows) captures the focused window in memory and asks a local vision model what’s on it. Nothing is saved unless you turn that on. Pull moondream: ollama pull moondream. macOS needs Screen Recording for Coda."
        />

        <Step
          done
          title="Open at login"
          body="LaunchAgent on macOS (not a LaunchDaemon, which would break screen capture). Windows uses a current-user Run key. Coda still hides instead of quitting."
        >
          <label className="flex items-center gap-2 text-xs">
            <Switch
              checked={Boolean(setup?.autostart)}
              onCheckedChange={(checked) => void setAutostart(checked)}
            />
            Start Coda when I sign in
          </label>
          <p className="text-[11px] text-muted-foreground">
            Folder watch: {setup?.watchFolder ?? "~/Downloads"} — new files that match a memory wait for your approval. Nothing is auto-opened.
          </p>
        </Step>

        {windows ? (
          <Step
            done
            title="Windows extras"
            body="WebView2 is bundled or downloaded on first launch. Microphone permission is requested when you record. Participant audio uses WASAPI loopback — nothing else to install."
          />
        ) : null}
      </ol>

      {setup?.whisperDirectory ? (
        <p className="mt-6 max-w-xl break-all text-[11px] text-muted-foreground">
          Model folder: {setup.whisperDirectory}
        </p>
      ) : null}
      {error ? <p className="mt-3 text-xs text-destructive">{error}</p> : null}

      <div className="mt-8 flex gap-2">
        <Button type="button" onClick={() => setView("meeting")}>
          Open meeting assistant
        </Button>
        <Button type="button" variant="secondary" onClick={() => setView("agent")}>
          Open agent workspace
        </Button>
      </div>
    </div>
  );
}

function Step({
  done,
  title,
  body,
  children,
}: {
  done: boolean;
  title: string;
  body: string;
  children?: ReactNode;
}) {
  return (
    <li className="rounded-lg border border-white/10 bg-black/20 p-4">
      <div className="flex items-start gap-3">
        {done ? (
          <CheckCircle2 className="mt-0.5 size-4 text-stealth" />
        ) : (
          <Circle className="mt-0.5 size-4 text-muted-foreground" />
        )}
        <div className="min-w-0 flex-1">
          <h2 className="text-sm font-medium">{title}</h2>
          <p className="mt-1 text-xs leading-relaxed text-muted-foreground">{body}</p>
          {children ? <div className="mt-3">{children}</div> : null}
        </div>
      </div>
    </li>
  );
}
