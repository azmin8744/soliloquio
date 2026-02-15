import { NaiveDateTime, UUID } from "./common.ts";

export interface User {
  id: UUID;
  email: string;
  createdAt?: NaiveDateTime;
  updatedAt?: NaiveDateTime;
}

export interface AuthorizedUser {
  token: string;
  refreshToken: string;
}

export interface PasswordChangeSuccess {
  message: string;
}

export interface ValidationErrorType {
  message: string;
}
