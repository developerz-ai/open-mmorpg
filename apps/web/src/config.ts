/**
 * Operator-brandable configuration. Realm name, accent, and feature flags come
 * from here (env-injected at deploy) — not code edits — mirroring the
 * data-driven core. An operator rebrands by setting env, never by forking.
 */
export interface OperatorConfig {
  realmName: string;
  gatewayUrl: string;
  features: {
    registrationOpen: boolean;
    cashShop: boolean;
  };
}

const env = import.meta.env;

export const config: OperatorConfig = {
  realmName: env.VITE_REALM_NAME ?? 'Open-MMORPG',
  gatewayUrl: env.VITE_GATEWAY_URL ?? 'http://localhost:8080',
  features: {
    registrationOpen: env.VITE_REGISTRATION_OPEN !== 'false',
    cashShop: env.VITE_CASH_SHOP === 'true',
  },
};
