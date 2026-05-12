function normalize(value: string): string {
  return value
    .toLowerCase()
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "");
}

function tokenMatches(token: string, text: string): boolean {
  if (!token) {
    return true;
  }

  return text.includes(token);
}

export function fuzzyMatch(query: string, text: string): boolean {
  const normalizedQuery = normalize(query).trim();
  if (!normalizedQuery) {
    return true;
  }

  const normalizedText = normalize(text);
  return normalizedQuery
    .split(/\s+/)
    .every((token) => tokenMatches(token, normalizedText));
}
