import type { IncidentItem } from "../types";

const categories = {
  HEA: { id: "HEA", key: "HEA", icon: "flame", color: "#d73a49", label: "Heating" },
  ELV: { id: "ELV", key: "ELV", icon: "arrow-up-down", color: "#0366d6", label: "Elevator" },
  PLB: { id: "PLB", key: "PLB", icon: "droplets", color: "#0e8a16", label: "Plumbing" },
  ELC: { id: "ELC", key: "ELC", icon: "zap", color: "#f9c513", label: "Electrical" },
} as const;

const invoicePdfUrl = new URL("../../common/assets/mock-files/invoice-2026-04.pdf", import.meta.url).href;
const elevatorInterventionPdfUrl = new URL(
  "../../common/assets/mock-files/elevator-intervention-2026-02.pdf",
  import.meta.url,
).href;
const elevatorCheckPhotoUrl = new URL(
  "../../common/assets/mock-files/elevator-check-photo.jpg",
  import.meta.url,
).href;

export const MOCK_INCIDENTS: IncidentItem[] = [
  {
    id: "INC-001",
    categoryCode: "HEA",
    category: categories.HEA,
    title: {
      en: "Heating outage on block B",
      fr: "Panne chauffage bloc B",
    },
    shortDescription: {
      en: "Boiler circuit pressure dropped overnight",
      fr: "Baisse de pression du circuit chaudiere pendant la nuit",
    },
    longDescription: {
      en: "A pressure fault triggered an automatic stop of the shared heating loop. Team is restoring circulation and checking valves.",
      fr: "Une anomalie de pression a provoque l'arret automatique de la boucle de chauffage collective. L'equipe retablit la circulation et controle les vannes.",
    },
    location: {
      en: "Boiler room - Building B",
      fr: "Chaufferie - Batiment B",
    },
    startUtc: "2020-05-01T07:00:00Z",
    endUtc: "2099-05-31T17:00:00Z",
    statusType: "ongoing",
    statusText: {
      en: "Installing replacement valve",
      fr: "Installation de la vanne de remplacement",
    },
    timeline: [
      {
        id: "INC-001-T3",
        atUtc: null,
        title: {
          en: "Awaiting valve replacement",
          fr: "Remplacement de vanne en attente",
        },
        details: {
          en: "Supplier confirmation is pending before final repair scheduling.",
          fr: "La confirmation fournisseur est attendue avant la planification de la reparation finale.",
        },
      },
      {
        id: "INC-001-T1",
        atUtc: "2020-05-01T07:05:00Z",
        title: {
          en: "Issue detected by monitoring system",
          fr: "Incident detecte par le systeme de supervision",
        },
        details: {
          en: "Temperature fell below threshold in two risers.",
          fr: "La temperature est passee sous le seuil sur deux colonnes.",
        },
      },
      {
        id: "INC-001-T2",
        atUtc: "2020-05-01T08:10:00Z",
        title: {
          en: "Technician dispatched",
          fr: "Technicien depose",
        },
      },
    ],
    attachments: [
      {
        id: "INC-001-A1",
        fileName: "invoice-2026-04.pdf",
        mimeType: "application/pdf",
        sizeBytes: 1210,
        url: invoicePdfUrl,
      },
    ],
  },
  {
    id: "INC-002",
    categoryCode: "ELV",
    category: categories.ELV,
    title: {
      en: "Elevator shutdown resolved",
      fr: "Arret ascenseur resolu",
    },
    shortDescription: {
      en: "Cabin controller rebooted after fault",
      fr: "Redemarrage du controleur cabine apres incident",
    },
    longDescription: {
      en: "The elevator experienced a control board fault and was restarted after safety checks.",
      fr: "L'ascenseur a subi un defaut de carte de controle et a ete relance apres controles de securite.",
    },
    location: {
      en: "Elevator shaft - Building A",
      fr: "Cage ascenseur - Batiment A",
    },
    startUtc: "2020-03-10T09:15:00Z",
    endUtc: "2020-03-10T10:05:00Z",
    statusType: "waiting",
    statusText: {
      en: "Resolved",
      fr: "Resolu",
    },
    timeline: [
      {
        id: "INC-002-T1",
        atUtc: "2020-03-10T09:15:00Z",
        title: {
          en: "Residents reported cabin stuck",
          fr: "Signalement cabine bloquee",
        },
      },
      {
        id: "INC-002-T2",
        atUtc: "2020-03-10T09:48:00Z",
        title: {
          en: "System reboot completed",
          fr: "Redemarrage systeme termine",
        },
      },
    ],
    attachments: [
      {
        id: "INC-002-A1",
        fileName: "elevator-intervention-2026-02.pdf",
        mimeType: "application/pdf",
        sizeBytes: 1324,
        url: elevatorInterventionPdfUrl,
      },
      {
        id: "INC-002-A2",
        fileName: "elevator-check-photo.jpg",
        mimeType: "image/jpeg",
        sizeBytes: 982,
        url: elevatorCheckPhotoUrl,
      },
    ],
  },
  {
    id: "INC-003",
    categoryCode: "ELC",
    category: categories.ELC,
    title: {
      en: "Generator alert cleared",
      fr: "Alerte generateur levee",
    },
    shortDescription: {
      en: "Backup generator alarm acknowledged",
      fr: "Alarme groupe electrogene acquittee",
    },
    longDescription: {
      en: "A transient voltage spike triggered a generator warning. Verification found no sustained issue.",
      fr: "Une pointe de tension transitoire a declenche une alerte du groupe electrogene. La verification n'a trouve aucun probleme durable.",
    },
    location: {
      en: "Technical room - Building C",
      fr: "Local technique - Batiment C",
    },
    startUtc: "2020-02-15T06:40:00Z",
    endUtc: "2020-02-15T07:20:00Z",
    statusType: "waiting",
    statusText: {
      en: "Awaiting review",
      fr: "En attente de verification",
    },
    timeline: [
      {
        id: "INC-003-T1",
        atUtc: "2020-02-15T06:40:00Z",
        title: {
          en: "Alert raised",
          fr: "Alerte declenchee",
        },
      },
      {
        id: "INC-003-T2",
        atUtc: "2020-02-15T07:20:00Z",
        title: {
          en: "Inspection completed",
          fr: "Inspection terminee",
        },
      },
    ],
    attachments: [],
  },
  {
    id: "INC-004",
    categoryCode: "PLB",
    category: categories.PLB,
    title: {
      en: "Water leakage in basement closed",
      fr: "Fuite d'eau en sous-sol cloturee",
    },
    shortDescription: {
      en: "Joint replaced on shared pipe",
      fr: "Joint remplace sur canalisation commune",
    },
    longDescription: {
      en: "A leak was isolated, damaged joint replaced, and pressure tests returned to nominal levels.",
      fr: "Une fuite a ete isolee, le joint endommage remplace, et les tests de pression sont revenus a la normale.",
    },
    location: {
      en: "Basement corridor",
      fr: "Couloir sous-sol",
    },
    startUtc: "2020-01-20T05:30:00Z",
    endUtc: "2020-01-20T09:10:00Z",
    statusType: "waiting",
    statusText: {
      en: "Resolved",
      fr: "Resolu",
    },
    timeline: [
      {
        id: "INC-004-T1",
        atUtc: "2020-01-20T05:30:00Z",
        title: {
          en: "Leak signal received",
          fr: "Signalement fuite recu",
        },
      },
      {
        id: "INC-004-T2",
        atUtc: "2020-01-20T08:45:00Z",
        title: {
          en: "Repair completed",
          fr: "Reparation terminee",
        },
      },
    ],
    attachments: [],
  },
];
