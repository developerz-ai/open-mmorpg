import type { JSX } from 'solid-js';
import type { Account } from '../lib/auth.ts';
import { PasswordSettingsForm } from './PasswordSettingsForm.tsx';
import { ProfileSettingsForm } from './ProfileSettingsForm.tsx';

/**
 * Account settings — profile (display name/email) and password change. Both are
 * PUT/POST intents to the gateway; the web holds no password hash. Rendered under
 * the read-only projection on the Account route. → docs/specs/web-client/account-auth
 */
export function AccountSettings(props: { account: Account }): JSX.Element {
  return (
    <>
      <ProfileSettingsForm account={props.account} />
      <PasswordSettingsForm />
    </>
  );
}
