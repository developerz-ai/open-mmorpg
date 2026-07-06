import { Alert, Button, Card } from '@omm/ui';
import type { JSX, ParentProps } from 'solid-js';
import { ErrorBoundary as SolidErrorBoundary } from 'solid-js';
import { t } from '../lib/i18n.ts';

export interface ErrorBoundaryProps extends ParentProps {
  /** Fallback UI component when an error occurs. */
  fallback?: (error: Error, retry: () => void) => JSX.Element;
  /** Error handler callback. */
  onError?: (error: Error) => void;
}

/**
 * Default error fallback with retry button.
 */
function DefaultFallback(props: { error: Error; retry: () => void }): JSX.Element {
  return (
    <Card title={t('error.title')}>
      <Alert tone="error">{t('error.message', { message: props.error.message })}</Alert>
      <div class="error-actions">
        <Button variant="primary" onClick={props.retry}>
          {t('error.retry')}
        </Button>
        <Button variant="ghost" onClick={() => window.location.reload()}>
          {t('error.refresh')}
        </Button>
      </div>
    </Card>
  );
}

/**
 * Error boundary — catches React/Solid errors in child tree and displays fallback.
 * Wrap routes or major features to prevent full-page crashes.
 */
export function ErrorBoundary(props: ErrorBoundaryProps): JSX.Element {
  return (
    <SolidErrorBoundary
      fallback={(err, reset) => {
        props.onError?.(err);
        const retry = (): void => {
          reset();
        };
        return props.fallback?.(err, retry) || <DefaultFallback error={err} retry={retry} />;
      }}
    >
      {props.children}
    </SolidErrorBoundary>
  );
}
