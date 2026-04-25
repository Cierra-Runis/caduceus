import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

import { Button } from '@/components/ui/button';

export default function Home() {
  const t = useTranslations('Home');
  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <div className='z-20 flex flex-col items-center justify-center gap-[18px]'>
        <div className='text-center leading-[1.2] font-bold tracking-tighter'>
          <div className={`bg-linear-91 bg-clip-text text-6xl text-primary`}>
            {t('caduceus')}
          </div>
        </div>
        <p className='text-center leading-7'>
          {t.rich('description', {
            typstApp: (chunks) => (
              <NextLink
                className='text-primary underline'
                href='https://typst.app'
              >
                {chunks}
              </NextLink>
            ),
          })}
        </p>
        <div className='flex items-center justify-center gap-6'>
          <Button asChild size='lg' variant='outline'>
            <NextLink href='/register'>{t('registerNow')}</NextLink>
          </Button>
          <Button asChild size='lg'>
            <NextLink href='https://github.com/Cierra-Runis/caduceus'>
              {t('viewOnGitHub')}
            </NextLink>
          </Button>
        </div>
      </div>
    </main>
  );
}
