import { Button } from '@heroui/button';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

export default function Home() {
  const t = useTranslations('Home');
  return (
    <section className='relative flex h-[calc(100vh_-_64px)] flex-col items-center justify-center overflow-hidden'>
      <section className='z-20 flex flex-col items-center justify-center gap-[18px] sm:gap-6'>
        <div className='text-center text-4xl leading-[1.2] font-bold tracking-tighter sm:text-6xl'>
          <div className='bg-linear-91 from-[hsl(var(--heroui-primary))] to-[hsl(var(--heroui-secondary))] bg-clip-text text-transparent'>
            Caduceus
          </div>
        </div>
        <p className='text-default-500 text-center leading-7 font-normal sm:w-[466px] sm:text-[18px]'>
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
      </section>
    </section>
  );
}
