export type LocaleCode = "en" | "fr";

export type LocalizedValue = {
  en?: string;
  fr?: string;
};

export function resolveLocalized(value: LocalizedValue, active: LocaleCode): string {
  return value[active] ?? value.en ?? value.fr ?? "";
}
