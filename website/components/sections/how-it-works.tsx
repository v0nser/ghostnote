const STAGES = [
  {
    n: "01",
    title: "Install the app",
    body: "Mac: GhostNote.dmg. Windows: GhostNote-Setup.exe (WebView2 downloads if needed). No account.",
    code: "GhostNote.dmg  or  GhostNote-Setup.exe",
  },
  {
    n: "02",
    title: "Finish Setup (especially on Windows)",
    body: "Open the Setup tab. Download the speech model in-app. Install Ollama and pull llama3.2:3b. WASAPI captures meeting audio — no extra driver.",
    code: "ollama pull llama3.2:3b",
  },
  {
    n: "03",
    title: "Meetings stay stealth",
    body: "Flip stealth. Join Zoom, Meet, or Teams. Ghost Note sits on your machine, not in theirs.",
    code: "stealth: on",
  },
  {
    n: "04",
    title: "Live answers",
    body: "VAD cuts the question. Whisper streams the line. A small local model writes one first-person reply.",
    code: "vad → whisper → ollama",
  },
  {
    n: "05",
    title: "Give the agent a goal",
    body: "Type or hold Voice. It plans, searches, drafts, and asks before anything sensitive.",
    code: "plan → tools → approval",
  },
  {
    n: "06",
    title: "Keep the trail",
    body: "Artifacts, memory, and an audit log stay local. Meeting action items can become tasks.",
    code: "summary → create task?",
  },
];

export function HowItWorks() {
  return (
    <section id="how" className="relative px-5 py-24">
      <div className="mx-auto max-w-6xl">
        <h2 className="text-3xl font-semibold md:text-4xl">
          From install to an agent in a few minutes
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
