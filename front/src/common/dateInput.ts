export function toDateTimeLocalInput(value?: string | null): string {
  return value ? value.slice(0, 16) : "";
}

export function toUtcFromDateTimeLocalInput(value: string): string | null {
  return value ? `${value}:00.000Z` : null;
}
