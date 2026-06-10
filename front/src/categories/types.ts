export type CategoryItem = {
  id: string;
  key: string;
  icon: string;
  color: string;
  label: string;
  labels: Record<string, string>;
};

export type CategoryInput = {
  key: string;
  icon: string;
  color: string;
  labels: Record<string, string>;
};
