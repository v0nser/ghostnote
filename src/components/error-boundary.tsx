import { Component, type ErrorInfo, type ReactNode } from "react";

type Props = { children: ReactNode };
type State = { error: string | null };

export class AppErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error: error.message || "Something went wrong." };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Coda UI error (app stays open)", error, info.componentStack);
  }

  render() {
    if (!this.state.error) {
      return this.props.children;
    }
    return (
      <div className="flex h-full w-full flex-col items-center justify-center gap-3 bg-[#121214] p-6 text-center text-sm text-foreground">
        <p className="text-xs uppercase tracking-[0.14em] text-muted-foreground">Coda is still running</p>
        <p>{this.state.error}</p>
        <button
          type="button"
          className="rounded-full bg-white/10 px-3 py-1.5 text-xs"
          onClick={() => this.setState({ error: null })}
        >
          Try again
        </button>
      </div>
    );
  }
}
