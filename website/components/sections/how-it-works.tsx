const STAGES = [
  {
    n: "01",
    title: "Download & Install",
    body: "One desktop app. No account. No cloud key.",
    code: "GhostNote.dmg  or  GhostNote-Setup.exe",
  },
  {
    n: "02",
    title: "Activate Stealth",
    body: "Flip the toggle. The window drops out of every screen share.",
    code: "stealth: on",
  },
  {
    n: "03",
    title: "Join Your Meeting",
    body: "Zoom, Meet, or Teams. GhostNote sits on your machine, not in theirs.",
    code: "capture: live",
  },
  {
    n: "04",
    title: "AI Listens & Learns",
    body: "VAD cuts the question the millisecond they stop. Whisper streams the line.",
    code: "vad → whisper",
  },
  {
    n: "05",
    title: "Get Instant Answers",
    body: "Ollama streams one first-person answer. You glance. You speak.",
    code: "stream: true",
  },
  {
    n: "06",
    title: "Ace Your Meeting",
    body: "Summarize when it ends. Walk out with the recap already written.",
    code: "summary ready",
  },
];

export function HowItWorks() {
  return (
    <section id="how" className="relative px-5 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-3xl font-semibold md:text-4xl">
          From Install to Intelligence in 60 Seconds
        </h2>
        <div className="mt-14 space-y-10">
          {STAGES.map((stage, index) => (
            <article
              key={stage.n}
              className="grid items-center gap-6 border-b border-white/10 pb-10 md:grid-cols-[140px_1fr_220px]"
            >
              <p className="font-mono text-5xl text-white/15">{stage.n}</p>
              <div>
                <h3 className="text-2xl font-semibold">{stage.title}</h3>
                <p className="mt-2 max-w-xl text-sm leading-relaxed text-mist">{stage.body}</p>
              </div>
              <pre className="rounded-xl border border-white/10 bg-charcoal px-4 py-3 font-mono text-xs text-cyan">
                {stage.code}
                {index === 0 ? <span className="ml-1 animate-pulse">▌</span> : null}
              </pre>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
