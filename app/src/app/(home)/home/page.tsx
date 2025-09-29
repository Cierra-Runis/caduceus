import { Button } from '@heroui/button';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

export default function Home() {
  const t = useTranslations('Home');
  return (
    <main className='flex flex-1 items-center justify-center px-6 py-16'>
      <div className='z-20 flex flex-col items-center justify-center gap-[18px]'>
        <div className='text-center leading-[1.2] font-bold tracking-tighter'>
          <div
            className={`
              bg-linear-91 from-[hsl(var(--heroui-primary))]
              to-[hsl(var(--heroui-secondary))] bg-clip-text text-6xl
              text-transparent
            `}
          >
            {t('caduceus')}
          </div>
        </div>
        <p className='text-center leading-7 text-foreground-500'>
          {t.rich('description', {
            typstApp: (chunks) => (
              <NextLink href='https://typst.app'>{chunks}</NextLink>
            ),
          })}
        </p>
        <div className='flex items-center justify-center gap-6'>
          <Button
            as={NextLink}
            color='primary'
            href='/register'
            size='lg'
            variant='ghost'
          >
            {t('registerNow')}
          </Button>
          <Button
            as={NextLink}
            href='https://github.com/Cierra-Runis/caduceus'
            size='lg'
            variant='bordered'
          >
            {t('viewOnGitHub')}
          </Button>
        </div>
      </div>
    </main>
  );
}
