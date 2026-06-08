import type { ProjectItem } from "../types";

export const MOCK_PROJECTS: ProjectItem[] = [
  {
    id: "PRJ/ONGOING?A",
    categoryCode: "GAR",
    category: { id: "GAR", code: "GAR", icon: "warehouse", color: "#6f42c1", label: "Garage" },
    title: { en: "Garage ventilation", fr: "Ventilation garage" },
    description: { en: "Fans are being installed.", fr: "Les ventilateurs sont en cours d'installation." },
    statusType: "ongoing",
    statusText: { en: "Installing fans", fr: "Installation des ventilateurs" },
    startUtc: undefined,
    endUtc: undefined,
    attachments: [],
  },
  {
    id: "PRJ/BIKE?1",
    categoryCode: "ELV",
    category: { id: "ELV", code: "ELV", icon: "arrow-up-down", color: "#0366d6", label: "Elevator" },
    title: { en: "Bike shelter", fr: "Abri velo" },
    description: {
      en: "# Bike shelter\n\nAdd a roof between the bike room and bins.",
      fr: "# Abri velo\n\nAjouter un toit entre le local velo et les poubelles.",
    },
    statusType: "waiting",
    statusText: { en: "Awaiting quote", fr: "Devis en attente" },
    startUtc: undefined,
    endUtc: undefined,
    attachments: [],
  },
  {
    id: "PRJ-ROOF-2",
    categoryCode: "HEA",
    category: { id: "HEA", code: "HEA", icon: "flame", color: "#d73a49", label: "Heating" },
    title: { en: "Roof insulation", fr: "Isolation toiture" },
    description: { en: "Insulation project completed.", fr: "Projet d'isolation termine." },
    statusType: "ongoing",
    statusText: { en: "Final inspection", fr: "Inspection finale" },
    startUtc: "2020-01-01T10:00:00Z",
    endUtc: "2020-02-01T10:00:00Z",
    attachments: [],
  },
];
