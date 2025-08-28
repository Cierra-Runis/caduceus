import Sidebar from '@/components/Sidebar';

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className='flex h-screen'>
      <Sidebar />
      <section className='flex h-full w-full flex-col p-4'>
        <header className='rounded-medium border-small border-divider flex flex-shrink-0 items-center gap-3 p-4'>
          <h2 className='text-medium text-default-700 font-medium'>Overview</h2>
        </header>
        <main className='mt-4 flex-1 overflow-auto'>
          <div className='rounded-medium border-small border-divider flex h-full w-full flex-col gap-4 p-4'>
            {children}
          </div>
        </main>
      </section>
    </div>
  );
}
