import { match } from '@formatjs/intl-localematcher';
import Negotiator from 'negotiator';
import { Locale } from 'next-intl';
import { getRequestConfig } from 'next-intl/server';
import { cookies, headers } from 'next/headers';

import message from '../../messages/en-US.json';

const AVAILABLE_LANGUAGES = ['en-US', 'jp-JP', 'zh-CN'];
const DEFAULT_LOCALE = 'en-US' as const;

declare module 'next-intl' {
  interface AppConfig {
    Locale: 'en-US' | 'ja-JP' | 'zh-CN';
    Messages: typeof message;
  }
}

export default getRequestConfig(async () => {
  // Check the language settings in the cookie first
  const cookieStore = await cookies();
  const localeCookie = cookieStore.get('locale')?.value;

  if (localeCookie && AVAILABLE_LANGUAGES.includes(localeCookie)) {
    return {
      locale: localeCookie as Locale,
      messages: (await import(`../../messages/${localeCookie}.json`)).default,
    };
  }

  // If there is no cookie, check headers
  const header = await headers();
  const acceptLanguage = header.get('accept-language');

  if (!acceptLanguage) {
    return {
      locale: DEFAULT_LOCALE,
      messages: (await import(`../../messages/${DEFAULT_LOCALE}.json`)).default,
    };
  }

  const accept = new Negotiator({
    headers: { 'accept-language': acceptLanguage },
  });
  const matchedLocale = match(
    accept.languages(),
    AVAILABLE_LANGUAGES,
    DEFAULT_LOCALE,
  );

  return {
    locale: matchedLocale as Locale,
    messages: (await import(`../../messages/${matchedLocale}.json`)).default,
  };
});
