import { useEffect, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { agentIpc, type AuditRow } from "@/lib/ipc/agent";
import { describeIpcError } from "@/lib/ipc/stealth";

export function AuditView() {
  const [query, setQuery] = useState("");
  const [rows, setRows] = useState<AuditRow[]>([]);
  const [error, setError] = useState<string | null>(null);

  const load = async (filter: string) => {
    try {
      const next = await agentIpc.listAudit(filter, 400);
      setRows(next);
      setError(null);
    } catch (err) {
      setError(describeIpcError(err));
    }
  };

  useEffect(() => {
    void load("");
  }, []);

  const filteredHint = useMemo(() => {
    if (!query.trim()) return `${rows.length} events`;
    return `${rows.length} matching “${query.trim()}”`;
  }, [query, rows.length]);

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden px-6 py-6">
      <div className="flex items-center justify-between gap-3">
        <div>
          <h1 className="text-lg font-medium">Audit</h1>
          <p className="mt-1 text-xs text-muted-foreground">
            Local log at ~/.coda/coda_audit.log. Nothing is uploaded.
          </p>
        </div>
        <Button
          type="button"
          size="sm"
          variant="secondary"
          onClick={() => {
            void (async () => {
              try {
                const text = await agentIpc.exportAudit();
                const blob = new Blob([text || ""], { type: "text/plain" });
                const url = URL.createObjectURL(blob);
                const a = document.createElement("a");
                a.href = url;
                a.download = "coda_audit.log";
                a.click();
                URL.revokeObjectURL(url);
              } catch (err) {
                setError(describeIpcError(err));
              }
            })();
          }}
        >
          Export
        </Button>
      </div>

      <form
        className="mt-4 flex gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          void load(query);
        }}
      >
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Filter by event or text"
          className="h-8 flex-1 rounded-md border border-white/10 bg-black/30 px-2 text-sm"
        />
        <Button type="submit" size="sm">
          Filter
        </Button>
      </form>
      <p className="mt-2 text-[11px] uppercase tracking-wide text-muted-foreground">{filteredHint}</p>
      {error ? <p className="mt-2 text-xs text-destructive">{error}</p> : null}

      <ol className="mt-3 min-h-0 flex-1 space-y-2 overflow-y-auto pr-1">
        {rows.map((row, index) => (
          <li key={`${row.timestamp}-${row.event}-${index}`} className="rounded-md border border-white/10 bg-black/20 px-3 py-2">
            <p className="text-[10px] uppercase tracking-wide text-muted-foreground">
              {new Date(row.timestamp).toLocaleString()} · {row.event}
            </p>
            <pre className="mt-1 overflow-x-auto whitespace-pre-wrap text-[11px] leading-relaxed text-foreground/80">
              {JSON.stringify(row.data, null, 2)}
            </pre>
          </li>
        ))}
      </ol>
    </div>
  );
}
