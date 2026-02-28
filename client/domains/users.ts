import { NaiveDateTime, UUID } from "./common.ts";

export interface User {
  id: UUID;
  email: string;
  emailVerifiedAt?: NaiveDateTime;
  displayName?: string;
  bio?: string;
  createdAt?: NaiveDateTime;
  updatedAt?: NaiveDateTime;
}

export interface ApiKey {
  id: UUID;
  label: string;
  lastUsedAt?: NaiveDateTime;
  createdAt: NaiveDateTime;
}

export interface CreateApiKeyResult {
  id: UUID;
  label: string;
  rawKey: string;
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
