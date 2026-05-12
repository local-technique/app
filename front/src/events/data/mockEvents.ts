import type { EventItem } from "../types";

const invoicePdfUrl = new URL("../../common/assets/mock-files/invoice-2026-04.pdf", import.meta.url).href;
const elevatorInterventionPdfUrl = new URL(
  "../../common/assets/mock-files/elevator-intervention-2026-02.pdf",
  import.meta.url,
).href;
const elevatorCheckPhotoUrl = new URL(
  "../../common/assets/mock-files/elevator-check-photo.jpg",
  import.meta.url,
).href;

export const MOCK_EVENTS: EventItem[] = [
  {
    id: "HEA-001",
    categoryCode: "HEA",
    title: {
      en: "Heating maintenance in progress",
      fr: "Maintenance chauffage en cours",
    },
    shortDescription: {
      en: "Boiler room preventive maintenance",
      fr: "Maintenance preventive chaufferie",
    },
    warning: {
      en: "no hot water between 9h30 & 17h00",
      fr: "pas d'eau chaude entre 9h30 et 17h00",
    },
    longDescription: {
      en: "Contractor is performing scheduled checks and balancing on the shared heating installation.",
      fr: "Le prestataire realise les controles planifies et l'equilibrage de l'installation de chauffage collective.",
    },
    location: {
      en: "Boiler room - Building B",
      fr: "Chaufferie - Batiment B",
    },
    startUtc: "2020-05-01T07:00:00Z",
    endUtc: "2099-05-31T17:00:00Z",
    notifiedAtUtc: "2020-04-29T10:00:00Z",
    handlers: ["Thermo Services"],
    attachments: [
      {
        id: "HEA-001-A1",
        fileName: "invoice-2026-04.pdf",
        mimeType: "application/pdf",
        sizeBytes: 1210,
        url: invoicePdfUrl,
      },
    ],
  },
  {
    id: "ELV-002",
    categoryCode: "ELV",
    title: {
      en: "Elevator annual maintenance",
      fr: "Maintenance annuelle ascenseur",
    },
    shortDescription: {
      en: "Planned intervention for safety checks",
      fr: "Intervention planifiee pour controle securite",
    },
    longDescription: {
      en: "The technician will perform annual maintenance and mandatory compliance checks.",
      fr: "Le technicien effectuera la maintenance annuelle et les controles reglementaires obligatoires.",
    },
    location: {
      en: "Elevator shaft - Building A",
      fr: "Cage ascenseur - Batiment A",
    },
    startUtc: "2099-06-12T08:30:00Z",
    notifiedAtUtc: "2099-05-28T09:00:00Z",
    handlers: ["LiftCare"],
    attachments: [
      {
        id: "ELV-002-A1",
        fileName: "elevator-intervention-2026-02.pdf",
        mimeType: "application/pdf",
        sizeBytes: 1324,
        url: elevatorInterventionPdfUrl,
      },
      {
        id: "ELV-002-A2",
        fileName: "elevator-check-photo.jpg",
        mimeType: "image/jpg",
        sizeBytes: 982,
        url: elevatorCheckPhotoUrl,
      },
    ],
  },
  {
    id: "PMG-003",
    categoryCode: "PMG",
    title: {
      en: "Property management site visit",
      fr: "Visite du syndic",
    },
    shortDescription: {
      en: "Follow-up visit with residents committee",
      fr: "Visite de suivi avec le conseil syndical",
    },
    longDescription: {
      en: "Property manager on-site visit to review current actions and upcoming works.",
      fr: "Visite sur site du syndic pour faire le point sur les actions en cours et les travaux a venir.",
    },
    location: {
      en: "Lobby - Building C",
      fr: "Hall - Batiment C",
    },
    startUtc: "2099-06-18T13:30:00Z",
    notifiedAtUtc: "2099-06-10T11:30:00Z",
    handlers: ["Gestion Plus"],
    attachments: [],
  },
  {
    id: "ELC-004",
    categoryCode: "ELC",
    title: {
      en: "Electrical panel inspection completed",
      fr: "Controle tableau electrique termine",
    },
    shortDescription: {
      en: "Routine inspection done",
      fr: "Inspection de routine terminee",
    },
    longDescription: {
      en: "Inspection confirmed no critical defects and issued standard recommendations.",
      fr: "L'inspection confirme l'absence de defaut critique et fournit des recommandations standard.",
    },
    location: {
      en: "Technical room - Building A",
      fr: "Local technique - Batiment A",
    },
    startUtc: "2020-02-15T08:00:00Z",
    endUtc: "2020-02-15T10:00:00Z",
    attachments: [],
  },
  {
    id: "PLB-005",
    categoryCode: "PLB",
    title: {
      en: "Pipe descaling intervention finished",
      fr: "Intervention detartrage canalisations terminee",
    },
    shortDescription: {
      en: "Vertical pipe descaling operation",
      fr: "Operation de detartrage des colonnes",
    },
    longDescription: {
      en: "Plumbing company completed descaling and pressure tests on shared pipes.",
      fr: "L'entreprise de plomberie a termine le detartrage et les tests de pression sur les canalisations communes.",
    },
    location: {
      en: "Basement technical corridor",
      fr: "Couloir technique sous-sol",
    },
    startUtc: "2020-03-21T06:30:00Z",
    endUtc: "2020-03-21T12:00:00Z",
    attachments: [],
  },
  {
    id: "GAR-006",
    categoryCode: "GAR",
    title: {
      en: "Garage door motor replacement completed",
      fr: "Remplacement moteur porte de garage termine",
    },
    shortDescription: {
      en: "Motor replaced after recurrent faults",
      fr: "Moteur remplace apres pannes recurrentes",
    },
    longDescription: {
      en: "The contractor replaced the motor and verified full opening and safety sensors.",
      fr: "Le prestataire a remplace le moteur et verifie l'ouverture complete ainsi que les capteurs de securite.",
    },
    location: {
      en: "Underground garage entrance",
      fr: "Entree garage sous-sol",
    },
    startUtc: "2020-01-28T09:00:00Z",
    endUtc: "2020-01-28T15:30:00Z",
    attachments: [],
  },
];
