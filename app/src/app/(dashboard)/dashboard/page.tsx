'use client';

import { Avatar } from '@heroui/avatar';
import { Button } from '@heroui/button';
import { Card, CardBody, CardFooter, CardHeader } from '@heroui/card';
import { Popover, PopoverContent, PopoverTrigger } from '@heroui/popover';
import { addToast } from '@heroui/toast';
import { redirect } from 'next/navigation';

export default function Dashboard() {
  return (
    <div className='space-y-6'>
      <div className='flex justify-between items-center'>
        <h1 className='text-3xl font-bold text-default-800'>Dashboard</h1>
        <Popover showArrow placement='bottom-end'>
          <PopoverTrigger>
            <Avatar
              src='https://avatars.githubusercontent.com/u/29329988'
              alt='user avatar'
            />
          </PopoverTrigger>
          <PopoverContent className='p-1'>
            <UserCard />
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}

export const UserCard = () => {
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
            <h4 className='text-small font-semibold leading-none text-default-600'>
              Zoey Lang
            </h4>
            <h5 className='text-small tracking-tight text-default-500'>
              @zoeylang
            </h5>
          </div>
        </div>
        <Button
          color='primary'
          radius='full'
          size='sm'
          onPress={() =>
            addToast({
              color: 'success',
              title: 'Signed out',
              description: 'You have been signed out successfully.',
              timeout: 3000,
              shouldShowTimeoutProgress: true,
              onClose: () => redirect('/'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
            })
          }
        >
          Sign Out
        </Button>
      </CardHeader>
      <CardBody className='px-3 py-0 overflow-clip'>
        <p className='text-small pl-px text-default-500'>
          Full-stack developer, @hero_ui lover she/her
          <span aria-label='confetti' role='img'>
            🎉
          </span>
        </p>
      </CardBody>
      <CardFooter className='gap-3'>
        <div className='flex gap-1'>
          <p className='font-semibold text-default-600 text-small'>4</p>
          <p className=' text-default-500 text-small'>Following</p>
        </div>
        <div className='flex gap-1'>
          <p className='font-semibold text-default-600 text-small'>97.1K</p>
          <p className='text-default-500 text-small'>Followers</p>
        </div>
      </CardFooter>
    </Card>
  );
};
