import { match } from '@formatjs/intl-localematcher';
import Negotiator from 'negotiator';
import { getRequestConfig } from 'next-intl/server';
import { cookies, headers } from 'next/headers';

export default getRequestConfig(async () => {
  const locales = ['en-US', 'zh-CN'];
  const defaultLocale = 'en-US';

  // Check the language settings in the cookie first
  const cookieStore = await cookies();
  const localeCookie = cookieStore.get('locale')?.value;

  if (localeCookie && locales.includes(localeCookie)) {
    return {
      locale: localeCookie,
      messages: (await import(`../../messages/${localeCookie}.json`)).default,
    };
  }

  // If there is no cookie, check headers
  const header = await headers();
  const acceptLanguage = header.get('accept-language');

  if (acceptLanguage) {
    const accept = new Negotiator({
      headers: { 'accept-language': acceptLanguage },
    });
    const matchedLocale = match(accept.languages(), locales, defaultLocale);

    return {
      locale: matchedLocale,
      messages: (await import(`../../messages/${matchedLocale}.json`)).default,
    };
  }

  // If no locale is found, use the default locale
  return {
    locale: defaultLocale,
    messages: (await import(`../../messages/${defaultLocale}.json`)).default,
  };
});
