import { NextIntlClientProvider } from 'next-intl';
import { ReactElement } from 'react';

import messages from '../../i18n/en-US.json';

/// Wrap a component tree in the en-US i18n provider for tests, so components
/// using `useTranslations` resolve without the Next.js server request config.
/// en-US strings equal the source labels, so queries stay readable.
export function withIntl(ui: ReactElement) {
  return (
    <NextIntlClientProvider locale='en-US' messages={messages}>
      {ui}
    </NextIntlClientProvider>
  );
}
