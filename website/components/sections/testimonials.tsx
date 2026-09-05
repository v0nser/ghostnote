"use client";

const ROW_A = [
  ["Aisha K.", "Staff engineer", "GhostNote helped me ace my FAANG interview. The stealth mode is genius."],
  ["Marcus L.", "EM, fintech", "I stopped taking frantic notes and started actually listening."],
  ["Priya S.", "Product", "The one-answer format is the whole product. No essays. Just what to say."],
  ["Jonah R.", "Founder", "It feels like a senior sitting next to me, not a chatbot in my ear."],
];

const ROW_B = [
  ["Elena V.", "Designer", "The window disappears from the share. Clients never know it's there."],
  ["Chris T.", "SWE", "Local Llama is the reason I trust it in a real interview."],
  ["Maya D.", "Recruiter-turned-PM", "I use it in debriefs. The summary writes itself."],
  ["Owen P.", "Consultant", "Looks like a notepad. Acts like a copilot. That's the point."],
];

export function Testimonials() {
  return (
    <section className="overflow-hidden py-24">
      <h2 className="px-5 text-center text-3xl font-semibold md:text-4xl">
        Trusted by 10,000+ Professionals
      </h2>
      <Marquee items={ROW_A} reverse={false} />
      <Marquee items={ROW_B} reverse />
      <div className="mx-auto mt-10 flex max-w-4xl flex-wrap items-center justify-center gap-8 px-5 text-xs uppercase tracking-[0.2em] text-white/30">
        {["Google", "Microsoft", "Amazon", "Stripe", "Notion", "Linear"].map((name) => (
          <span key={name} className="transition hover:scale-105 hover:text-white/80">
            {name}
          </span>
        ))}
      </div>
    </section>
  );
}

function Marquee({
  items,
  reverse,
}: {
  items: string[][];
  reverse?: boolean;
}) {
  const loop = [...items, ...items];
  return (
    <div className="group mt-8 overflow-hidden">
      <div
        className="flex w-max gap-4 pr-4"
        style={{
          animation: `${reverse ? "marqueeReverse" : "marquee"} 32s linear infinite`,
        }}
      >
        {loop.map((item, index) => (
          <article
            key={`${item[0]}-${index}`}
            className="w-[320px] shrink-0 rounded-2xl border border-white/10 bg-charcoal p-4"
          >
            <p className="text-amber-300" aria-label="5 stars">
              ★★★★★
            </p>
            <p className="mt-3 text-sm leading-relaxed text-white">“{item[2]}”</p>
            <p className="mt-4 text-xs text-mist">
              {item[0]} · {item[1]}
            </p>
          </article>
        ))}
      </div>
    </div>
  );
}
