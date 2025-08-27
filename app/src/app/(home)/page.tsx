import { Button } from '@heroui/button';
import NextLink from 'next/link';

export default function Home() {
  return (
    <section className='relative flex flex-col items-center justify-center h-[calc(100vh_-_64px)] overflow-hidden'>
      <section className='z-20 flex flex-col items-center justify-center gap-[18px] sm:gap-6'>
        <div className='text-center text-4xl leading-[1.2] font-bold tracking-tighter sm:text-6xl'>
          <div className='bg-linear-91 from-[hsl(var(--heroui-primary))] to-[hsl(var(--heroui-secondary))] bg-clip-text text-transparent'>
            Caduceus
          </div>
        </div>
        <p className='text-default-500 text-center leading-7 font-normal sm:w-[466px] sm:text-[18px]'>
          An open-source alternative to{' '}
          <NextLink href='https://typst.app'>Typst App</NextLink>.
        </p>
        <div className='flex items-center justify-center gap-6'>
          <Button
            href='/register'
            as={NextLink}
            color='primary'
            variant='ghost'
            size='lg'
          >
            Register now!
          </Button>
          <Button
            href='https://github.com/Cierra-Runis/caduceus'
            as={NextLink}
            variant='bordered'
            size='lg'
          >
            View on GitHub
          </Button>
        </div>
      </section>
    </section>
  );
}
