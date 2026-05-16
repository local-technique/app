export function sanitizeRedirectPath(value: string): string {
  if (!value || !value.startsWith("/")) {
    return "/events";
  }

  if (value.startsWith("//")) {
    return "/events";
  }

  if (value.includes("\\")) {
    return "/events";
  }

  for (const char of value) {
    if (/\p{C}/u.test(char)) {
      return "/events";
    }
  }

  return value;
}
