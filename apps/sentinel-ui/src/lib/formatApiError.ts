import { SentinelError } from "@sentinel/auth-sdk";

/**
 * Extracts a human-readable error message from an API error.
 *
 * For ValidationError responses the server returns a `details` object keyed
 * by field name, each containing an array of validator error entries. This
 * function flattens those into a readable string, e.g.:
 *   "rules: rules cannot be empty"
 *
 * Falls back to `err.message` for any other error type.
 */
export function formatApiError(err: unknown): string {
  if (err instanceof SentinelError && err.details != null && typeof err.details === "object") {
    const msgs: string[] = [];
    for (const [field, errors] of Object.entries(err.details as Record<string, unknown>)) {
      if (!Array.isArray(errors)) continue;
      for (const e of errors) {
        if (typeof e !== "object" || e === null) continue;
        const entry = e as Record<string, unknown>;
        const text =
          typeof entry.message === "string" && entry.message
            ? entry.message
            : typeof entry.code === "string"
              ? entry.code
              : null;
        if (text) msgs.push(`${field}: ${text}`);
      }
    }
    if (msgs.length > 0) return msgs.join("; ");
    // details present but unparseable — fall through to message
    return err.message;
  }
  return (err as Error)?.message ?? "An unexpected error occurred.";
}
