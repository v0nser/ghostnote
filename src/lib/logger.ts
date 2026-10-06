/**
 * The only sanctioned logging path in the frontend.
 *
 * Coda handles meeting transcripts, summaries and notes. Anything written
 * to the console is readable by anyone who opens devtools, and would also be
 * visible on a shared screen. So:
 *
 * - In release builds every level below `error` compiles down to a no-op.
 * - Even `error` only ever takes a static message, never user content.
 *
 * Call `console.*` directly nowhere else.
 */

const isDev = import.meta.env.DEV;

const noop = () => {};

export const log = {
  debug: isDev ? console.debug.bind(console, "[coda]") : noop,
  info: isDev ? console.info.bind(console, "[coda]") : noop,
  warn: isDev ? console.warn.bind(console, "[coda]") : noop,

  /**
   * Reports a failure. Pass a fixed, human-written message — never a
   * transcript, note body, prompt or model response.
   */
  error: (message: string) => console.error("[coda]", message),
};
