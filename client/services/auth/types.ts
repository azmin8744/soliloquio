import {
  ApiKey,
  AuthorizedUser,
  CreateApiKeyResult,
  EmailVerifySuccess,
  PasswordChangeSuccess,
  PasswordResetSuccess,
  User,
  ValidationErrorType,
} from "../../domains/users.ts";
import { AuthError, DbError } from "../../domains/common.ts";

export interface SignUpInput {
  email: string;
  password: string;
}

export interface SignInInput {
  email: string;
  password: string;
}

export interface ChangePasswordInput {
  currentPassword: string;
  newPassword: string;
}

export interface UpdateUserInput {
  email: string;
  displayName?: string;
  bio?: string;
}

export type UserMutationResult =
  | AuthorizedUser
  | ValidationErrorType
  | DbError
  | AuthError
  | PasswordChangeSuccess
  | PasswordResetSuccess
  | EmailVerifySuccess
  | User
  | CreateApiKeyResult
  | { id: string };

export type { ApiKey, CreateApiKeyResult };
