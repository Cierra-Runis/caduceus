import { useTranslations } from 'next-intl';

// This page renders when a route like `/unknown.txt` is requested.
// In this case, the layout at `app/layout.tsx` receives.
export default function GlobalNotFound() {
  const t = useTranslations();
  return (
    <div className='flex h-screen items-center justify-center'>
      <div className='flex flex-col items-center text-center'>
        <div className='text-4xl font-bold'>404</div>
        <div className='mt-4 text-lg'>{t('404.description')}</div>
      </div>
    </div>
  );
}
