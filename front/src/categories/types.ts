export type CategoryItem = {
  id: string;
  code: string;
  icon: string;
  color: string;
  label: string;
  labels: Record<string, string>;
};

export type CategoryInput = {
  id?: string;
  code: string;
  icon: string;
  color: string;
  labels: Record<string, string>;
};
