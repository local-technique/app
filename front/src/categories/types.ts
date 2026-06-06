export type CategoryItem = {
  id: string;
  code: string;
  icon: string;
  label: string;
  labels: Record<string, string>;
};

export type CategoryInput = {
  id?: string;
  code: string;
  icon: string;
  labels: Record<string, string>;
};
