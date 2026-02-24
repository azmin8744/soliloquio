import { NaiveDateTime, UUID } from "./common.ts";

export interface User {
  id: UUID;
  email: string;
  emailVerifiedAt?: NaiveDateTime;
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

export interface PasswordResetSuccess {
  message: string;
}

export interface EmailVerifySuccess {
  message: string;
}

export interface ValidationErrorType {
  message: string;
}
