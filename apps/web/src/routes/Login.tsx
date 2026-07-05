import { Alert, Button, Card, TextField } from '@omm/ui';
import { A, useNavigate } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { createSignal, Show } from 'solid-js';
import { authMessageKey } from '../lib/auth.ts';
import { t } from '../lib/i18n.ts';
import { useLogin } from '../queries/useAuth.ts';

/** Login route — thin: collects credentials, submits the intent, renders result. */
export default function Login(): JSX.Element {
  const navigate = useNavigate();
  const mutation = useLogin();
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');

  const submit = (e: Event): void => {
    e.preventDefault();
    mutation.mutate(
      { email: email(), password: password() },
      { onSuccess: () => navigate('/account') },
    );
  };

  return (
    <Card title={t('auth.login.heading')} class="stack">
      <form class="auth-form" onSubmit={submit}>
        <Show when={mutation.isError}>
          <Alert tone="error">{t(authMessageKey(mutation.error))}</Alert>
        </Show>
        <TextField
          id="login-email"
          type="email"
          autocomplete="email"
          required
          label={t('auth.login.email')}
          value={email()}
          onInput={(e) => setEmail(e.currentTarget.value)}
        />
        <TextField
          id="login-password"
          type="password"
          autocomplete="current-password"
          required
          label={t('auth.login.password')}
          value={password()}
          onInput={(e) => setPassword(e.currentTarget.value)}
        />
        <Button type="submit" variant="primary" disabled={mutation.isPending}>
          {t('auth.login.submit')}
        </Button>
      </form>
      <p class="text-fg-muted">
        {t('auth.login.noAccount')} <A href="/register">{t('auth.login.register')}</A>
      </p>
    </Card>
  );
}
