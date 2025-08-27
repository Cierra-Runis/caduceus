'use client';

import { Button } from '@heroui/button';
import { Card, CardBody, CardHeader } from '@heroui/card';
import { Divider } from '@heroui/divider';
import { Input } from '@heroui/input';
import { Select, SelectItem } from '@heroui/select';
import { Switch } from '@heroui/switch';
import {
  IconBell,
  IconPalette,
  IconShield,
  IconUser,
} from '@tabler/icons-react';

export default function SettingsPage() {
  return (
    <div className='container mx-auto space-y-6'>
      {/* Header */}
      <div>
        <h1 className='text-default-800 text-3xl font-bold'>Settings</h1>
        <p className='text-default-600 mt-1'>
          Manage your account settings and preferences
        </p>
      </div>

      {/* Profile Settings */}
      <Card>
        <CardHeader className='pb-3'>
          <div className='flex items-center gap-2'>
            <IconUser size={20} />
            <h3 className='text-default-800 text-lg font-semibold'>Profile</h3>
          </div>
        </CardHeader>
        <CardBody className='space-y-4'>
          <div className='grid grid-cols-1 gap-4 md:grid-cols-2'>
            <Input
              defaultValue='John Doe'
              label='Name'
              placeholder='Enter your name'
            />
            <Input
              defaultValue='john.doe@example.com'
              label='Email'
              placeholder='Enter your email'
              type='email'
            />
          </div>
          <Input
            defaultValue='System Administrator'
            label='Bio'
            placeholder='Tell us about yourself'
          />
          <div className='flex justify-end'>
            <Button color='primary'>Save Changes</Button>
          </div>
        </CardBody>
      </Card>

      {/* Notification Settings */}
      <Card>
        <CardHeader className='pb-3'>
          <div className='flex items-center gap-2'>
            <IconBell size={20} />
            <h3 className='text-default-800 text-lg font-semibold'>
              Notifications
            </h3>
          </div>
        </CardHeader>
        <CardBody className='space-y-4'>
          <div className='space-y-4'>
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-default-800 font-medium'>
                  Email Notifications
                </p>
                <p className='text-default-500 text-sm'>
                  Receive email notifications for important updates
                </p>
              </div>
              <Switch defaultSelected />
            </div>
            <Divider />
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-default-800 font-medium'>
                  Push Notifications
                </p>
                <p className='text-default-500 text-sm'>
                  Receive push notifications in your browser
                </p>
              </div>
              <Switch defaultSelected />
            </div>
            <Divider />
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-default-800 font-medium'>System Updates</p>
                <p className='text-default-500 text-sm'>
                  Notify me when system updates are available
                </p>
              </div>
              <Switch />
            </div>
          </div>
        </CardBody>
      </Card>

      {/* Appearance Settings */}
      <Card>
        <CardHeader className='pb-3'>
          <div className='flex items-center gap-2'>
            <IconPalette size={20} />
            <h3 className='text-default-800 text-lg font-semibold'>
              Appearance
            </h3>
          </div>
        </CardHeader>
        <CardBody className='space-y-4'>
          <div className='grid grid-cols-1 gap-4 md:grid-cols-2'>
            <Select
              defaultSelectedKeys={['light']}
              label='Theme'
              placeholder='Select theme'
            >
              <SelectItem key='light'>Light Theme</SelectItem>
              <SelectItem key='dark'>Dark Theme</SelectItem>
              <SelectItem key='auto'>Follow System</SelectItem>
            </Select>
            <Select
              defaultSelectedKeys={['en']}
              label='Language'
              placeholder='Select language'
            >
              <SelectItem key='zh-cn'>简体中文</SelectItem>
              <SelectItem key='en'>English</SelectItem>
              <SelectItem key='ja'>日本語</SelectItem>
            </Select>
          </div>
        </CardBody>
      </Card>

      {/* Security Settings */}
      <Card>
        <CardHeader className='pb-3'>
          <div className='flex items-center gap-2'>
            <IconShield size={20} />
            <h3 className='text-default-800 text-lg font-semibold'>Security</h3>
          </div>
        </CardHeader>
        <CardBody className='space-y-4'>
          <div className='space-y-4'>
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-default-800 font-medium'>
                  Two-Factor Authentication
                </p>
                <p className='text-default-500 text-sm'>
                  Add an extra layer of security to your account
                </p>
              </div>
              <Button size='sm' variant='flat'>
                Setup
              </Button>
            </div>
            <Divider />
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-default-800 font-medium'>Change Password</p>
                <p className='text-default-500 text-sm'>
                  Update your password regularly to keep your account secure
                </p>
              </div>
              <Button size='sm' variant='flat'>
                Change
              </Button>
            </div>
            <Divider />
            <div className='flex items-center justify-between'>
              <div>
                <p className='text-default-800 font-medium'>Login Devices</p>
                <p className='text-default-500 text-sm'>
                  View and manage your logged-in devices
                </p>
              </div>
              <Button size='sm' variant='flat'>
                View
              </Button>
            </div>
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
