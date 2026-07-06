import { useMutation, useQuery, useQueryClient } from '@tanstack/solid-query';
import {
  type Account,
  type ChangePasswordInput,
  changePassword,
  fetchAccount,
  type LoginInput,
  login,
  logout,
  type RegisterInput,
  register,
  type UpdateProfileInput,
  updateProfile,
} from '../lib/auth.ts';
import { sessionToken } from '../lib/session.ts';

const ACCOUNT_KEY = ['account'] as const;

/** The current account projection — only queried while a session token exists. */
export function useAccount() {
  return useQuery<Account>(() => ({
    queryKey: ACCOUNT_KEY,
    queryFn: fetchAccount,
    enabled: sessionToken() !== null,
    staleTime: 60_000,
  }));
}

/** Login mutation; refreshes account state on success. */
export function useLogin() {
  const qc = useQueryClient();
  return useMutation(() => ({
    mutationFn: (input: LoginInput) => login(input),
    onSuccess: (s) => qc.setQueryData(ACCOUNT_KEY, s.account),
  }));
}

/** Register mutation; refreshes account state on success. */
export function useRegister() {
  const qc = useQueryClient();
  return useMutation(() => ({
    mutationFn: (input: RegisterInput) => register(input),
    onSuccess: (s) => qc.setQueryData(ACCOUNT_KEY, s.account),
  }));
}

/** Logout mutation; clears account state. */
export function useLogout() {
  const qc = useQueryClient();
  return useMutation(() => ({
    mutationFn: () => logout(),
    onSuccess: () => qc.removeQueries({ queryKey: ACCOUNT_KEY }),
  }));
}

/** Profile-update mutation; refreshes the cached account projection on success. */
export function useUpdateProfile() {
  const qc = useQueryClient();
  return useMutation(() => ({
    mutationFn: (input: UpdateProfileInput) => updateProfile(input),
    onSuccess: (account: Account) => qc.setQueryData(ACCOUNT_KEY, account),
  }));
}

/** Password-change mutation; a typed error surfaces the gateway code. */
export function useChangePassword() {
  return useMutation(() => ({
    mutationFn: (input: ChangePasswordInput) => changePassword(input),
  }));
}
