/** All visitor-facing copy. Edit here; components import from this file. */

export type Who = "engineer" | "friend" | "coda";
export type Mood = "tired" | "confused" | "curious" | "calm" | "happy" | "asleep";
export type PropId = "rec" | "tabs" | "sticky" | "cal" | "pdf";
export type PropState = "chaos" | "settling" | "resolved" | "dock";
export type StepKind = "input" | "model" | "local" | "tool" | "gate" | "log";

export interface Line {
  who: Who;
  text: string;
  /** Desk items that turn from mess into finished work when this line starts. */
  resolves?: PropId[];
}

export interface Chapter {
  id: string;
  num: string;
  title: string;
  caption: string;
  props: PropState;
  coda: boolean;
  moods: Record<Who, Mood>;
  lines: Line[];
}

export interface TraceStep {
  label: string;
  where: string;
  detail: string;
  kind: StepKind;
}

export interface Route {
  id: string;
  cue: string;
  summary: string;
  steps: TraceStep[];
}

export const cast: Record<Who, { name: string; role: string; label: string }> = {
  engineer: {
    name: "Kabir",
    role: "the engineer",
    label: "Kabir, an engineer in a dark hoodie with messy hair",
  },
  friend: {
    name: "Ria",
    role: "the friend",
    label: "Ria, his friend, with round glasses and a hair bun",
  },
  coda: {
    name: "Coda",
    role: "the stagehand",
    label: "Coda, a small marigold orb with a leaf sprout and pill-shaped eyes",
  },
};

export const site = {
  name: "Coda",
  tagline: "Coda: Your work, resolved. Your mind, at peace.",
  download: {
    label: "Download Coda",
    subtitle: "Runs locally. Requires Ollama.",
    href: "https://github.com/v0nser/ghostnote",
  },
  source: { label: "Read the source", href: "https://github.com/v0nser/ghostnote" },
  nav: [
    { href: "#story", label: "Story" },
    { href: "#machine", label: "Machine" },
    { href: "#try", label: "Try it" },
    { href: "#faq", label: "FAQ" },
    { href: "#build", label: "Build it" },
  ],

  hero: {
    kicker: "cold open",
    title: "Your work, resolved.",
    italic: "Your mind, at peace.",
    skip: "Skip intro",
    enter: "Watch the story",
    download: "Download Coda",
    open: [
      { who: "coda" as const, text: "You're early. They're still drowning." },
      { who: "friend" as const, text: "Who are you talking to?" },
      { who: "coda" as const, text: "The person holding the laptop. Hi." },
      { who: "engineer" as const, text: "There's a recording. Nobody is going to watch it." },
      { who: "coda" as const, text: "I live here. I prepare the work. You say yes. Scroll — I'll show you." },
    ],
  },

  story: {
    chapters: [
      {
        id: "confusion",
        num: "01",
        title: "The Confusion",
        caption: "A recording nobody watched. Forty-seven tabs. A sticky note that just says URGENT.",
        props: "chaos",
        coda: false,
        moods: { engineer: "tired", friend: "confused", coda: "calm" },
        lines: [
          { who: "engineer", text: "Did anyone write down who owns the action items?" },
          { who: "friend", text: "I don't even know which tab the PDF is in." },
          { who: "engineer", text: "There's a recording. Nobody is going to watch it." },
          { who: "friend", text: "And two meetings at two o'clock. Somehow." },
        ],
      },
      {
        id: "arrival",
        num: "02",
        title: "The Arrival",
        caption: "Something small and warm floats in from the edge of the desk.",
        props: "settling",
        coda: true,
        moods: { engineer: "curious", friend: "curious", coda: "calm" },
        lines: [
          { who: "friend", text: "…what is that?" },
          { who: "coda", text: "Hi. I'm Coda. I live on this laptop — nowhere else." },
          { who: "engineer", text: "Can you deal with this mess?" },
          { who: "coda", text: "I can prepare all of it. You decide what happens." },
        ],
      },
      {
        id: "resolve",
        num: "03",
        title: "The Resolve",
        caption: "One cue at a time. Each thing becomes finished work — or a draft waiting for a yes.",
        props: "resolved",
        coda: true,
        moods: { engineer: "happy", friend: "curious", coda: "happy" },
        lines: [
          { who: "coda", text: "The recording: three action items, with owners and due dates.", resolves: ["rec"] },
          { who: "coda", text: "You said “plea my focus playlist”. I heard play.", resolves: ["tabs"] },
          { who: "coda", text: "The recap email is drafted. It waits for your yes.", resolves: ["sticky"] },
          { who: "friend", text: "Wait — it didn't send it?", resolves: ["cal", "pdf"] },
          { who: "coda", text: "Never on my own. You're the director. I'm the stagehand." },
        ],
      },
      {
        id: "quiet",
        num: "04",
        title: "The Quiet",
        caption: "The desk is clear. Everything that needs a decision is waiting in one place.",
        props: "dock",
        coda: true,
        moods: { engineer: "calm", friend: "asleep", coda: "calm" },
        lines: [
          { who: "engineer", text: "…that's it?" },
          { who: "coda", text: "That's it. Everything I did is in the audit log." },
          { who: "engineer", text: "Huh. My mind is actually quiet." },
        ],
      },
    ] satisfies Chapter[],
  },

  machine: {
    eyebrow: "The nervous system",
    title: "A cue has a body.",
    titleItalic: "Watch it move.",
    intro:
      "Coda is not a cloud. It is a small set of nerves on this laptop: an ear, an eye, a cortex, a hand, and a conscience that is you. Simple cues skip the cortex. Risky ones stop at the gate. The cloud is not plugged in.",
    chassis: "this laptop",
    cloud: "cloud gpt",
    organs: [
      { id: "ear", name: "Ear", model: "Whisper small.en", note: "Hears. Never streams." },
      { id: "eye", name: "Eye", model: "moondream", note: "Looks once. Throws the frame away." },
      { id: "router", name: "Reflex", model: "Fast intent", note: "Play, open, find — no planning." },
      { id: "cortex", name: "Cortex", model: "llama3.1:8b", note: "Plans. Drafts. Does not send." },
      { id: "hand", name: "Hand", model: "Mail, Music, Files", note: "Moves only after a yes." },
      { id: "you", name: "You", model: "the director", note: "The only one who can let it go." },
    ],
    routes: [
      {
        id: "play",
        cue: "“plea the focus playlist”",
        summary: "Spoken, misheard, corrected, done — without waking the planner.",
        steps: [
          { kind: "input", label: "Microphone", where: "this laptop", detail: "Audio is captured locally. It is never streamed anywhere." },
          { kind: "model", label: "Whisper", where: "small.en · whisper-cli sidecar", detail: "Transcribes on-device with beam search. It heard “plea the focus playlist”." },
          { kind: "local", label: "Correction", where: "on-device rules", detail: "“plea” before a music phrase becomes “play”. “please help” is left alone." },
          { kind: "local", label: "Fast intent", where: "intent router", detail: "Simple commands skip the language model entirely. No plan, no wait." },
          { kind: "tool", label: "Music", where: "Apple Music", detail: "Opens the Focus playlist and starts playback." },
          { kind: "log", label: "Verify + log", where: "~/.coda/coda_audit.log", detail: "Coda checks playback actually started before saying so, then writes a plain-English line." },
        ],
      },
      {
        id: "email",
        cue: "“send the recap to Alex”",
        summary: "Planned by a local model, drafted, then stopped — until you approve.",
        steps: [
          { kind: "input", label: "Your words", where: "Agent tab", detail: "Typed or spoken. Same path either way." },
          { kind: "model", label: "Planner", where: "llama3.1:8b · Ollama on localhost", detail: "Breaks the goal into steps: find the recap, draft the email, ask before sending." },
          { kind: "tool", label: "Draft", where: "local draft", detail: "Writes the email with recipient, subject and body. Does not send." },
          { kind: "gate", label: "Your yes", where: "approval card", detail: "You see exactly what will go out. Approve, edit, or deny. Nothing moves until you choose." },
          { kind: "tool", label: "Mail", where: "Mail / Outlook on this machine", detail: "Only after approval, sent from your own mail app." },
          { kind: "log", label: "Log", where: "~/.coda/coda_audit.log", detail: "The draft, your decision and the result are recorded." },
        ],
      },
      {
        id: "screen",
        cue: "⌘⇧C  “what's on my screen?”",
        summary: "A frame held in memory, read by a small vision model, then thrown away.",
        steps: [
          { kind: "input", label: "Hotkey", where: "⌘⇧C · Ctrl+Shift+C", detail: "Captures the focused window before Coda's own window appears." },
          { kind: "local", label: "Frame in RAM", where: "JPEG, memory only", detail: "Downscaled and kept in memory. Not written to disk unless you turn that on." },
          { kind: "local", label: "Fast intent", where: "intent router", detail: "Skips the planner and goes straight to vision." },
          { kind: "model", label: "moondream", where: "vision model · Ollama", detail: "Reads the frame and streams an answer into a small overlay." },
          { kind: "local", label: "Discard", where: "Esc", detail: "Close the overlay and the frame is gone." },
        ],
      },
      {
        id: "forget",
        cue: "“forget my manager's name”",
        summary: "Shows you every match first. Deletes for real — only after you confirm.",
        steps: [
          { kind: "input", label: "Your words", where: "Agent tab", detail: "“forget …” is recognised before any planning happens." },
          { kind: "model", label: "Memory search", where: "nomic-embed-text · local", detail: "Finds memories and open meeting items that match, using local embeddings." },
          { kind: "gate", label: "Your confirm", where: "approval card", detail: "Every match is listed. Nothing is deleted yet." },
          { kind: "tool", label: "Delete", where: "~/.coda/agent", detail: "Rows and their embeddings are removed. Not hidden — gone." },
          { kind: "log", label: "Log", where: "~/.coda/coda_audit.log", detail: "A receipt that the forget happened." },
        ],
      },
    ] satisfies Route[],
  },

  principles: {
    eyebrow: "Principles",
    items: [
      { big: "No cloud GPT.", small: "Whisper and Ollama run on localhost. If they're off, Coda waits. It doesn't quietly phone a server." },
      { big: "No silent auto-send.", small: "Email, calendar events, reminders, deletes — each one stops at a card with your decision on it." },
      { big: "A receipt for everything.", small: "Every step lands in ~/.coda/coda_audit.log in plain English. Filter it, export it, keep it." },
    ],
  },

  demo: {
    eyebrow: "Try it",
    title: "A rehearsal,",
    titleItalic: "not the show.",
    disclaimer: "Simulated demo. The real app runs entirely on your machine — this page calls no models.",
    empty: "Cue something. Pick a chip below or type one of them.",
    presets: [
      { id: "voice", label: "plea the focus playlist" },
      { id: "screen", label: "what's on my screen?" },
      { id: "meeting", label: "summarize the standup" },
      { id: "approval", label: "send the recap to alex@example.com" },
      { id: "forget", label: "forget that I mentioned my manager's name" },
    ],
  },

  honest: {
    eyebrow: "Honest limits",
    title: "What Coda won't pretend.",
    items: [
      "No cloud models. If Ollama is down, Coda waits — it does not phone home.",
      "Scanned-image PDFs aren't read. Text PDFs are.",
      "Speech recognition is Whisper small.en. Good, not perfect. “plea” is mapped to “play” for music, and that's the kind of fix it needs.",
      "You need Ollama and a capable machine. llama3.1:8b plans; moondream answers screen questions.",
    ],
  },

  faq: {
    eyebrow: "Questions",
    title: "Asked, answered.",
    items: [
      {
        q: "Does any audio or text leave my computer?",
        a: "No. Microphone audio goes to a local Whisper sidecar. Planning uses Ollama on localhost. The audit log is a file in ~/.coda.",
      },
      {
        q: "Will Coda send email or create calendar events on its own?",
        a: "No. They stay drafts until you approve them in the app. There is no silent auto-send.",
      },
      {
        q: "What happened to Ghost Note?",
        a: "Coda is the rename. Your Ghost Note data is copied into ~/.coda on first launch. The old folder is not deleted.",
      },
      {
        q: "What do I install besides Coda?",
        a: "Ollama, then pull llama3.1:8b — and optionally moondream and nomic-embed-text. Whisper small.en installs from the in-app Setup tab.",
      },
    ],
  },

  contribute: {
    eyebrow: "Build it with us",
    title: "Coda is open source.",
    titleItalic: "Make it stronger.",
    intro:
      "Every line of Coda is on GitHub. If it mishears you, misses a file, or could be kinder to your laptop, you can fix it — or tell us where it hurts.",
    repo: "v0nser/ghostnote",
    botName: "Patch",
    botLabel: "Patch, a small ink robot with antenna ears, a wrench hand and a branch-shaped tail",
    pokes: [
      "Hi, I'm Patch. I keep the pull requests tidy.",
      "Found a bug? I want to hear about it. Really.",
      "Whisper heard “plea” once. Somebody fixed it. That could be you next.",
      "No PR is too small. Typos count.",
      "Coda gets stronger one yes at a time. Mine is waiting for yours.",
    ],
    ways: [
      {
        id: "bug",
        title: "Report what broke",
        body: "A misheard command, a tool that said “done” but wasn't, a crash. Steps to reproduce are gold.",
        cta: "File an issue",
        path: "/issues/new",
      },
      {
        id: "code",
        title: "Fix an open issue",
        body: "Rust in src-tauri, React in src. Pick an issue, say you're on it, open a PR against main.",
        cta: "Browse issues",
        path: "/issues",
      },
      {
        id: "voice",
        title: "Sharpen the ear",
        body: "Speech corrections live in transcribe/correct.rs. Add a word Whisper keeps mishearing, with a test.",
        cta: "Read correct.rs",
        path: "/blob/main/src-tauri/src/transcribe/correct.rs",
      },
      {
        id: "tool",
        title: "Teach it a tool",
        body: "New fast intents and tools must verify their result and stop for approval when they touch the outside world.",
        cta: "See the agent",
        path: "/tree/main/src-tauri/src/agent",
      },
      {
        id: "art",
        title: "Redraw the cast",
        body: "Kabir, Ria, Coda and Patch are first-pass SVG. Hand-ink them and keep the layer names.",
        cta: "Read ASSETS.md",
        path: "/blob/main/coda-site/ASSETS.md",
      },
      {
        id: "docs",
        title: "Make setup easier",
        body: "Windows steps, Ollama models, permissions after the rename. If it confused you, it confuses others.",
        cta: "Open the docs",
        path: "/tree/main/docs",
      },
    ],
    steps: [
      { label: "Fork & clone", code: "git clone https://github.com/v0nser/ghostnote.git" },
      { label: "Install", code: "npm install" },
      { label: "Run the app", code: "npm run tauri:dev" },
      { label: "Test", code: "cd src-tauri && cargo test --lib" },
      { label: "Open a PR", code: "git push origin my-fix" },
    ],
    rules: [
      "Local only. No cloud model calls, ever.",
      "Nothing sends, deletes or books without an approval card.",
      "Tools verify what they did before they say “done”.",
    ],
    primary: "Star & fork on GitHub",
    secondary: "File an issue",
    live: { stars: "stars", forks: "forks", issues: "open issues" },
  },

  rest: {
    title: "Your work, resolved.",
    italic: "Your mind, at peace.",
  },

  footer: {
    note: "Coda is local-first software. This page is a drawing — no user counts, no benchmarks, no testimonials.",
  },
};
