import { Alert, Button, Card, TextField } from '@omm/ui';
import { A, useNavigate } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createSignal, Show } from 'solid-js';
import { authMessageKey } from '../lib/auth.ts';
import { isEnabled } from '../lib/features.ts';
import { t } from '../lib/i18n.ts';
import { useRegister } from '../queries/useAuth.ts';

/** Registration route. The form is also gated server-side; this only offers it. */
export default function Register(): JSX.Element {
  const navigate = useNavigate();
  const mutation = useRegister();
  const [displayName, setDisplayName] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');

  const submit = (e: Event): void => {
    e.preventDefault();
    mutation.mutate(
      { displayName: displayName(), email: email(), password: password() },
      { onSuccess: () => navigate('/account') },
    );
  };

  return (
    <Show
      when={isEnabled('registrationOpen')}
      fallback={<Alert tone="info">{t('auth.register.closed')}</Alert>}
    >
      <Card title={t('auth.register.heading')} class="stack">
        <form class="auth-form" onSubmit={submit}>
          <Show when={mutation.isError}>
            <Alert tone="error">{t(authMessageKey(mutation.error))}</Alert>
          </Show>
          <TextField
            id="reg-name"
            autocomplete="nickname"
            required
            label={t('auth.register.displayName')}
            value={displayName()}
            onInput={(e) => setDisplayName(e.currentTarget.value)}
          />
          <TextField
            id="reg-email"
            type="email"
            autocomplete="email"
            required
            label={t('auth.register.email')}
            value={email()}
            onInput={(e) => setEmail(e.currentTarget.value)}
          />
          <TextField
            id="reg-password"
            type="password"
            autocomplete="new-password"
            required
            minLength={8}
            label={t('auth.register.password')}
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
          />
          <Button type="submit" variant="primary" disabled={mutation.isPending}>
            {t('auth.register.submit')}
          </Button>
        </form>
        <p class="text-fg-muted">
          {t('auth.register.haveAccount')} <A href="/login">{t('auth.register.login')}</A>
        </p>
      </Card>
    </Show>
  );
}
