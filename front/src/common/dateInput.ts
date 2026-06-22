export function toDateTimeLocalInput(value?: string | null): string {
  return value ? value.slice(0, 16) : "";
}

export function toUtcFromDateTimeLocalInput(value: string): string | null {
  return value ? `${value}:00.000Z` : null;
}

export function toDateLocalInput(value?: string | null): string {
  return value ? value.slice(0, 10) : "";
}

export function toUtcFromDateLocalInput(value: string, now: Date = new Date()): string | null {
  if (!value) return null;
  const time = `${String(now.getHours()).padStart(2, "0")}:${String(now.getMinutes()).padStart(2, "0")}:00.000Z`;
  return `${value}T${time}`;
}

export function todayDateInput(): string {
  const d = new Date();
  const yyyy = d.getFullYear();
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}
