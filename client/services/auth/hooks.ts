import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  changePassword,
  forgotPassword,
  getMe,
  resendVerificationEmail,
  resetPassword,
  signIn,
  signUp,
  updateUser,
  verifyEmail,
} from "./api.ts";
import {
  ChangePasswordInput,
  SignInInput,
  SignUpInput,
  UpdateUserInput,
} from "./types.ts";
import { authKeys } from "./keys.ts";

export function useSignUp() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SignUpInput) => signUp(input),
    onSuccess: (data) => {
      // Backend sets httpOnly cookies, just invalidate user query
      if ("token" in data) {
        queryClient.invalidateQueries({ queryKey: authKeys.user() });
      }
    },
    onError: (error) => {
      console.log(error);
    },
  });
}

export function useSignIn() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: SignInInput) => signIn(input),
    onSuccess: (data) => {
      // Backend sets httpOnly cookies, just invalidate user query
      if ("token" in data) {
        queryClient.invalidateQueries({ queryKey: authKeys.user() });
      }
    },
    onError: (error) => {
      console.log(error);
    },
  });
}

export function useChangePassword() {
  return useMutation({
    mutationFn: (input: ChangePasswordInput) => changePassword(input),
  });
}

export function useUpdateUser() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: UpdateUserInput) => updateUser(input),
    onSuccess: (data) => {
      if ("email" in data) {
        queryClient.invalidateQueries({ queryKey: authKeys.user() });
      }
    },
  });
}

export function useLogout() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async () => {
      // Call BFF logout endpoint to clear cookies + revoke token
      await fetch("/api/auth/logout", {
        method: "POST",
        credentials: "include",
      });
    },
    onSuccess: () => {
      queryClient.setQueryData(authKeys.user(), null);
      queryClient.invalidateQueries({ queryKey: authKeys.user() });
    },
  });
}

export function useForgotPassword() {
  return useMutation({
    mutationFn: (email: string) => forgotPassword(email),
  });
}

export function useResetPassword() {
  return useMutation({
    mutationFn: (
      { token, newPassword }: { token: string; newPassword: string },
    ) => resetPassword(token, newPassword),
  });
}

export function useVerifyEmail() {
  return useMutation({
    mutationFn: (token: string) => verifyEmail(token),
  });
}

export function useResendVerificationEmail() {
  return useMutation({
    mutationFn: () => resendVerificationEmail(),
  });
}

export function useMe() {
  return useQuery({
    queryKey: authKeys.user(),
    queryFn: async () => {
      return await getMe();
    },
    retry: false, // Don't retry if 401/403 or token invalid
  });
}
