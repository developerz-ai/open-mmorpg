import { Alert, Button, Card, TextField } from '@omm/ui';
import type { JSX } from 'solid-js';
import { createSignal, Show } from 'solid-js';
import { type Account, authMessageKey } from '../lib/auth.ts';
import { t } from '../lib/i18n.ts';
import { useChangePassword, useUpdateProfile } from '../queries/useAuth.ts';

/**
 * Account settings — profile (display name/email) and password change. Both are
 * PUT/POST intents to the gateway; the web holds no password hash. Rendered under
 * the read-only projection on the Account route. → docs/specs/web-client/account-auth
 */
export function AccountSettings(props: { account: Account }): JSX.Element {
  const profile = useUpdateProfile();
  const password = useChangePassword();

  const [displayName, setDisplayName] = createSignal(props.account.displayName);
  const [email, setEmail] = createSignal(props.account.email);
  const [profileSaved, setProfileSaved] = createSignal(false);

  const [currentPassword, setCurrentPassword] = createSignal('');
  const [newPassword, setNewPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');
  const [mismatch, setMismatch] = createSignal(false);
  const [passwordChanged, setPasswordChanged] = createSignal(false);

  const saveProfile = (e: Event): void => {
    e.preventDefault();
    setProfileSaved(false);
    profile.mutate(
      { displayName: displayName().trim(), email: email().trim() },
      { onSuccess: () => setProfileSaved(true) },
    );
  };

  const submitPassword = (e: Event): void => {
    e.preventDefault();
    setPasswordChanged(false);
    if (newPassword() !== confirmPassword()) {
      setMismatch(true);
      return;
    }
    setMismatch(false);
    password.mutate(
      { currentPassword: currentPassword(), newPassword: newPassword() },
      {
        onSuccess: () => {
          setPasswordChanged(true);
          setCurrentPassword('');
          setNewPassword('');
          setConfirmPassword('');
        },
      },
    );
  };

  return (
    <>
      <Card title={t('auth.account.settings.profileHeading')} class="stack">
        <form class="auth-form" onSubmit={saveProfile}>
          <Show when={profile.isError}>
            <Alert tone="error">{t(authMessageKey(profile.error))}</Alert>
          </Show>
          <Show when={profileSaved()}>
            <Alert tone="success">{t('auth.account.settings.profileSaved')}</Alert>
          </Show>
          <TextField
            id="settings-display-name"
            label={t('auth.account.displayName')}
            value={displayName()}
            onInput={(e) => {
              setDisplayName(e.currentTarget.value);
              setProfileSaved(false);
            }}
          />
          <TextField
            id="settings-email"
            type="email"
            autocomplete="email"
            label={t('auth.account.email')}
            value={email()}
            onInput={(e) => {
              setEmail(e.currentTarget.value);
              setProfileSaved(false);
            }}
          />
          <Button type="submit" variant="primary" disabled={profile.isPending}>
            {t('auth.account.settings.saveProfile')}
          </Button>
        </form>
      </Card>

      <Card title={t('auth.account.settings.passwordHeading')} class="stack">
        <form class="auth-form" onSubmit={submitPassword}>
          <Show when={password.isError}>
            <Alert tone="error">{t(authMessageKey(password.error))}</Alert>
          </Show>
          <Show when={mismatch()}>
            <Alert tone="error">{t('auth.error.password_mismatch')}</Alert>
          </Show>
          <Show when={passwordChanged()}>
            <Alert tone="success">{t('auth.account.settings.passwordChanged')}</Alert>
          </Show>
          <TextField
            id="settings-current-password"
            type="password"
            autocomplete="current-password"
            required
            label={t('auth.account.settings.currentPassword')}
            value={currentPassword()}
            onInput={(e) => {
              setCurrentPassword(e.currentTarget.value);
              setPasswordChanged(false);
              setMismatch(false);
            }}
          />
          <TextField
            id="settings-new-password"
            type="password"
            autocomplete="new-password"
            required
            minLength={8}
            label={t('auth.account.settings.newPassword')}
            value={newPassword()}
            onInput={(e) => {
              setNewPassword(e.currentTarget.value);
              setPasswordChanged(false);
              setMismatch(false);
            }}
          />
          <TextField
            id="settings-confirm-password"
            type="password"
            autocomplete="new-password"
            required
            minLength={8}
            label={t('auth.account.settings.confirmPassword')}
            value={confirmPassword()}
            onInput={(e) => {
              setConfirmPassword(e.currentTarget.value);
              setPasswordChanged(false);
              setMismatch(false);
            }}
          />
          <Button type="submit" variant="primary" disabled={password.isPending}>
            {t('auth.account.settings.changePassword')}
          </Button>
        </form>
      </Card>
    </>
  );
}
