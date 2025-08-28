import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

// This page renders when a route like `/unknown.txt` is requested.
// In this case, the layout at `app/layout.tsx` receives.
export default function GlobalNotFound() {
  const t = useTranslations();
  return (
    <main className='flex min-h-screen items-center justify-center'>
      <div className='flex flex-col items-center text-center'>
        <NextLink className='text-4xl font-bold' href='/'>
          404
        </NextLink>
        <div className='mt-4 text-lg'>{t('404.description')}</div>
      </div>
    </main>
  );
}
