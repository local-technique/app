export type TimeStatus = "TO_COME" | "ONGOING" | "PAST";

export type DisplayStatus = "waiting" | "ongoing" | "finished" | "planned";

export function computeDisplayStatus(
  stored: "waiting" | "ongoing",
  timeStatus: TimeStatus,
): DisplayStatus {
  if (timeStatus === "PAST") return "finished";
  if (stored === "ongoing" && timeStatus === "TO_COME") return "planned";
  return stored;
}

export function computeTimeStatus(
  startUtc?: string | null,
  endUtc?: string | null,
  now = new Date(),
): TimeStatus {
  const nowMs = now.getTime();
  const startMs = startUtc ? Date.parse(startUtc) : null;
  const endMs = endUtc ? Date.parse(endUtc) : null;

  if (startMs === null) {
    if (endMs !== null && endMs <= nowMs) return "PAST";
    return "TO_COME";
  }

  if (startMs > nowMs) return "TO_COME";

  if (endMs === null) return "ONGOING";

  if (endMs <= nowMs) return "PAST";

  return "ONGOING";
}
