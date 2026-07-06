import { Alert, Button, Card, TextField } from '@omm/ui';
import type { JSX } from 'solid-js';
import { createSignal, Show } from 'solid-js';
import type { Account } from '../lib/auth.ts';
import { authMessageKey } from '../lib/auth.ts';
import { t } from '../lib/i18n.ts';
import { useUpdateProfile } from '../queries/useAuth.ts';

export function ProfileSettingsForm(props: { account: Account }): JSX.Element {
  const profile = useUpdateProfile();

  const [displayName, setDisplayName] = createSignal(props.account.displayName);
  const [email, setEmail] = createSignal(props.account.email);
  const [profileSaved, setProfileSaved] = createSignal(false);

  const saveProfile = (e: Event): void => {
    e.preventDefault();
    setProfileSaved(false);
    profile.mutate(
      { displayName: displayName().trim(), email: email().trim() },
      { onSuccess: () => setProfileSaved(true) },
    );
  };

  return (
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
  );
}
