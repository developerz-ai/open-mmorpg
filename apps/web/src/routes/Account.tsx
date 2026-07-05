import { Alert, Button, Card, Spinner } from '@omm/ui';
import { A, useNavigate } from '@solidjs/router';
import type { JSX } from 'solid-js';
import { Match, Switch } from 'solid-js';
import { fmt, t } from '../lib/i18n.ts';
import { sessionToken } from '../lib/session.ts';
import { useAccount, useLogout } from '../queries/useAuth.ts';

/** Account management — renders the account projection; logout revokes the session. */
export default function Account(): JSX.Element {
  const navigate = useNavigate();
  const account = useAccount();
  const logout = useLogout();

  if (sessionToken() === null) {
    return (
      <Card title={t('auth.account.heading')}>
        <p class="text-fg-muted">{t('auth.account.loginPrompt')}</p>
        <A href="/login">
          <Button variant="primary">{t('nav.login')}</Button>
        </A>
      </Card>
    );
  }

  return (
    <Card title={t('auth.account.heading')} class="stack">
      <Switch>
        <Match when={account.isPending}>
          <Spinner label={t('common.loading')} />
        </Match>
        <Match when={account.isError}>
          <Alert tone="error">{t('errors.unknown')}</Alert>
        </Match>
        <Match when={account.data}>
          {(data) => (
            <dl class="stack">
              <div>
                <dt class="text-fg-muted">{t('auth.account.displayName')}</dt>
                <dd class="text-fg-strong">{data().displayName}</dd>
              </div>
              <div>
                <dt class="text-fg-muted">{t('auth.account.email')}</dt>
                <dd>{data().email}</dd>
              </div>
              <p class="text-fg-muted">
                {t('auth.account.memberSince', { date: fmt.date(new Date(data().createdAt)) })}
              </p>
            </dl>
          )}
        </Match>
      </Switch>
      <Button
        variant="ghost"
        disabled={logout.isPending}
        onClick={() => logout.mutate(undefined, { onSuccess: () => navigate('/') })}
      >
        {t('auth.account.logout')}
      </Button>
    </Card>
  );
}
