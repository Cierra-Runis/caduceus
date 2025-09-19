export type ApiResponse<T> = {
  message: string;
  payload: T;
};

export type ErrorResponse = ApiResponse<undefined>;
