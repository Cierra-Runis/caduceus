export interface AuthPayload {
  token: string;
  user: UserPayload;
}
export interface UserPayload {
  createAt: Date;
  id: string;
  nickname: string;
  updateAt: Date;
  username: string;
}
