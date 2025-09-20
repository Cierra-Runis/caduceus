export interface AuthPayload {
  token: string;
  user: UserPayload;
}
export interface UserPayload {
  avatar_uri?: string;
  createAt: Date;
  id: string;
  nickname: string;
  updateAt: Date;
  username: string;
}
