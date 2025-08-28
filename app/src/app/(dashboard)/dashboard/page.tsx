'use client';

import { Avatar } from '@heroui/avatar';
import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { addToast } from '@heroui/toast';
import { redirect } from 'next/navigation';

export default function Dashboard() {
  return (
    <main className='space-y-6'>
      {/* <div className='flex items-center justify-between'>
        <h1 className='text-default-800 text-3xl font-bold'>Dashboard</h1>
        <Popover placement='bottom-end' showArrow>
          <PopoverTrigger>
            <Avatar
              alt='user avatar'
              src='https://avatars.githubusercontent.com/u/29329988'
            />
          </PopoverTrigger>
          <PopoverContent className='p-1'>
            <UserCard />
          </PopoverContent>
        </Popover>
      </div> */}
    </main>
  );
}

const UserCard = () => {
  return (
    <Card className='max-w-[300px] border-none bg-transparent' shadow='none'>
      <CardHeader className='justify-between'>
        <div className='flex gap-3'>
          <Avatar
            isBordered
            radius='full'
            size='md'
            src='https://i.pravatar.cc/150?u=a04258114e29026702d'
          />
          <div className='flex flex-col items-start justify-center'>
            <h4 className='text-small text-default-600 leading-none font-semibold'>
              Zoey Lang
            </h4>
            <h5 className='text-small text-default-500 tracking-tight'>
              @zoeylang
            </h5>
          </div>
        </div>
        <Button
          color='primary'
          onPress={() =>
            addToast({
              color: 'success',
              description: 'You have been signed out successfully.',
              // FIXME: https://github.com/heroui-inc/heroui/issues/5609
              onClose: async () => redirect('/logout'),
              shouldShowTimeoutProgress: true,
              timeout: 3000,
              title: 'Signed out',
            })
          }
          radius='full'
          size='sm'
        >
          Sign Out
        </Button>
      </CardHeader>
      <CardBody className='overflow-clip px-3 py-0'>
        <p className='text-small text-default-500 pl-px'>
          Full-stack developer, @hero_ui lover she/her
          <span aria-label='confetti' role='img'>
            🎉
          </span>
        </p>
      </CardBody>
      <CardFooter className='gap-3'>
        <div className='flex gap-1'>
          <p className='text-default-600 text-small font-semibold'>4</p>
          <p className=' text-default-500 text-small'>Following</p>
        </div>
        <div className='flex gap-1'>
          <p className='text-default-600 text-small font-semibold'>97.1K</p>
          <p className='text-default-500 text-small'>Followers</p>
        </div>
      </CardFooter>
    </Card>
  );
};
