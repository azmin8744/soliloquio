// GraphQL queries are just strings, no special library needed
import { getGraphQLClient } from "../../utils/graphql_client.ts";
import {
  ChangePasswordInput,
  SignInInput,
  SignUpInput,
  UpdateUserInput,
  UserMutationResult,
} from "./types.ts";
import { User } from "../../domains/users.ts";

const SIGN_UP_MUTATION = `
  mutation SignUp($input: SignUpInput!) {
    signUp(input: $input) {
      ... on AuthorizedUser {
        token
        refreshToken
      }
      ... on ValidationErrorType {
        message
      }
      ... on DbError {
        message
      }
      ... on AuthError {
        message
      }
    }
  }
`;

const SIGN_IN_MUTATION = `
  mutation SignIn($input: SignInInput!) {
    signIn(input: $input) {
      ... on AuthorizedUser {
        token
        refreshToken
      }
      ... on ValidationErrorType {
        message
      }
      ... on DbError {
        message
      }
      ... on AuthError {
        message
      }
    }
  }
`;

const CHANGE_PASSWORD_MUTATION = `
  mutation ChangePassword($input: ChangePasswordInput!) {
    changePassword(input: $input) {
      ... on PasswordChangeSuccess {
        message
      }
      ... on ValidationErrorType {
        message
      }
      ... on DbError {
        message
      }
      ... on AuthError {
        message
      }
    }
  }
`;

const UPDATE_USER_MUTATION = `
  mutation UpdateUser($input: UpdateUserInput!) {
    updateUser(input: $input) {
      ... on User { id email }
      ... on ValidationErrorType { message }
      ... on AuthError { message }
      ... on DbError { message }
    }
  }
`;

const ME_QUERY = `
  query Me {
    me {
      id
      email
      createdAt
      updatedAt
    }
  }
`;

export async function signUp(input: SignUpInput): Promise<UserMutationResult> {
  const client = getGraphQLClient();
  const data = await client.request<{ signUp: UserMutationResult }>(
    SIGN_UP_MUTATION,
    { input },
  );
  return data.signUp;
}

export async function signIn(input: SignInInput): Promise<UserMutationResult> {
  const client = getGraphQLClient();
  const data = await client.request<{ signIn: UserMutationResult }>(
    SIGN_IN_MUTATION,
    { input },
  );
  return data.signIn;
}

export async function changePassword(
  input: ChangePasswordInput,
): Promise<UserMutationResult> {
  const client = getGraphQLClient();
  const data = await client.request<{ changePassword: UserMutationResult }>(
    CHANGE_PASSWORD_MUTATION,
    { input },
  );
  return data.changePassword;
}

export async function updateUser(input: UpdateUserInput): Promise<UserMutationResult> {
  const client = getGraphQLClient();
  const data = await client.request<{ updateUser: UserMutationResult }>(
    UPDATE_USER_MUTATION,
    { input },
  );
  return data.updateUser;
}

export async function getMe(): Promise<User | null> {
  const client = getGraphQLClient();
  try {
    const data = await client.request<{ me: User }>(ME_QUERY);
    return data.me;
  } catch (_error) {
    return null;
  }
}
