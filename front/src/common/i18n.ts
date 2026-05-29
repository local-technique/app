import { createI18n } from "vue-i18n";

export const FALLBACK_LOCALES = ["en", "fr"] as const;
export type LocaleCode = "en" | "fr";
export type LocalePreference = LocaleCode | "system";
const STORAGE_KEY = "copro-locale";

export const messages = {
  en: {
    nav: {
      events: "Events & Maintenance",
      incidents: "Incidents",
    },
    labels: {
      current: "Current",
      toCome: "To come",
      past: "Past",
      searchEvents: "Search events",
      searchIncidents: "Search incidents",
      noEventsMatch: "No events match your search.",
      noIncidentsMatch: "No incidents match your search.",
      eventsLoadFailed: "Events could not be loaded. Please sign in again or retry later.",
      incidentsLoadFailed: "Incidents could not be loaded. Please sign in again or retry later.",
      backToEvents: "Back to events",
      backToIncidents: "Back to incidents",
      eventNotFound: "Event not found",
      incidentNotFound: "Incident not found",
      eventLoadFailed: "Event could not be loaded. Please sign in again or retry later.",
      incidentLoadFailed: "Incident could not be loaded. Please sign in again or retry later.",
      incidentTimeline: "Incident timeline",
      attachments: "Attachments",
      noAttachments: "No attachments available.",
      preview: "Preview",
      previewUnavailable: "Preview unavailable for this file type.",
      pdfPreviewUnavailable: "PDF preview is unavailable.",
      openFile: "Open file",
      pageNotFound: "Page not found",
      routeMissing: "This route does not exist.",
      openMore: "Open more",
      warningPrefix: "⚠️",
    },
    controls: {
      language: "Language",
      theme: "Theme",
      menu: "Menu",
      openMenu: "Open menu",
      closeMenu: "Close menu",
    },
    options: {
      system: "System",
      light: "Light",
      dark: "Dark",
    },
  },
  fr: {
    nav: {
      events: "Évènements et maintenance",
      incidents: "Incidents",
    },
    labels: {
      current: "En cours",
      toCome: "À venir",
      past: "Passé",
      searchEvents: "Rechercher des évènements",
      searchIncidents: "Rechercher des incidents",
      noEventsMatch: "Aucun évènement ne correspond à votre recherche.",
      noIncidentsMatch: "Aucun incident ne correspond à votre recherche.",
      eventsLoadFailed: "Les évènements n'ont pas pu être chargés. Veuillez vous reconnecter ou réessayer plus tard.",
      incidentsLoadFailed: "Les incidents n'ont pas pu être chargés. Veuillez vous reconnecter ou réessayer plus tard.",
      backToEvents: "Retour aux évènements",
      backToIncidents: "Retour aux incidents",
      eventNotFound: "Évènement introuvable",
      incidentNotFound: "Incident introuvable",
      eventLoadFailed: "L'évènement n'a pas pu être chargé. Veuillez vous reconnecter ou réessayer plus tard.",
      incidentLoadFailed: "L'incident n'a pas pu être chargé. Veuillez vous reconnecter ou réessayer plus tard.",
      incidentTimeline: "Chronologie de l'incident",
      attachments: "Pièces jointes",
      noAttachments: "Aucune pièce jointe disponible.",
      preview: "Aperçu",
      previewUnavailable: "Aperçu indisponible pour ce type de fichier.",
      pdfPreviewUnavailable: "L'aperçu PDF est indisponible.",
      openFile: "Ouvrir le fichier",
      pageNotFound: "Page introuvable",
      routeMissing: "Cette route n'existe pas.",
      openMore: "Ouvrir plus",
      warningPrefix: "⚠️",
    },
    controls: {
      language: "Langue",
      theme: "Thème",
      menu: "Menu",
      openMenu: "Ouvrir le menu",
      closeMenu: "Fermer le menu",
    },
    options: {
      system: "Système",
      light: "Clair",
      dark: "Sombre",
    },
  },
} as const;

export function getSystemLocale(): LocaleCode {
  if (typeof navigator === "undefined") {
    return "fr";
  }

  const first = (navigator.languages && navigator.languages[0]) || navigator.language || "fr";
  const normalized = first.toLowerCase();
  return normalized.startsWith("fr") ? "fr" : "en";
}

export function resolveLocalePreference(preference: LocalePreference): LocaleCode {
  if (preference === "system") {
    return getSystemLocale();
  }
  return preference;
}

export function getStoredLocalePreference(storage: Pick<Storage, "getItem"> = localStorage): LocalePreference {
  const value = storage.getItem(STORAGE_KEY);
  if (value === "en" || value === "fr" || value === "system") {
    return value;
  }
  return "system";
}

export function getStoredLocale(storage: Pick<Storage, "getItem"> = localStorage): LocaleCode {
  return resolveLocalePreference(getStoredLocalePreference(storage));
}

export function setStoredLocale(locale: LocalePreference, storage: Pick<Storage, "setItem"> = localStorage): void {
  storage.setItem(STORAGE_KEY, locale);
}

export function createAppI18n(initialLocale: LocaleCode = "fr") {
  return createI18n({
    legacy: false,
    locale: initialLocale,
    fallbackLocale: [...FALLBACK_LOCALES],
    messages,
  });
}
