/**
 * Tauri commands reject with the backend's ErrorPayload shape
 * ({ code, message, details }), but plain strings/Errors can also show up.
 */
export function getErrorMessage(err: unknown, fallback = "Something went wrong"): string {
  if (!err) return fallback;
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  if (typeof err === "object" && "message" in err && typeof (err as any).message === "string") {
    return (err as any).message;
  }
  return fallback;
}
