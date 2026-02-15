import {
  AuthorizedUser,
  PasswordChangeSuccess,
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

export type UserMutationResult =
  | AuthorizedUser
  | ValidationErrorType
  | DbError
  | AuthError
  | PasswordChangeSuccess;
