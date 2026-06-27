type ApiUserRef = {
  id: string;
  email: string;
  first_name?: string | null;
  last_name?: string | null;
};

type UserRef = {
  id: string;
  email: string;
  firstName?: string | null;
  lastName?: string | null;
};

export function mapUserRef(api: ApiUserRef | null | undefined): UserRef | null {
  if (!api) return null;
  return { id: api.id, email: api.email, firstName: api.first_name, lastName: api.last_name };
}
