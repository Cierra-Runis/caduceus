export type ProjectPayload = {
  created_at: Date;
  creator_id: string;
  id: string;
  name: string;
  owner_id: string;
  owner_type: 'team' | 'user';
  updated_at: Date;
};
