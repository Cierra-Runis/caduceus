"use client";

import { Button } from '@heroui/button';
import { Card, CardBody, CardHeader } from '@heroui/card';
import { Divider } from '@heroui/divider';
import { Input } from '@heroui/input';
import { Select, SelectItem } from '@heroui/select';
import { Switch } from '@heroui/switch';
import { IconBell, IconPalette, IconShield, IconUser } from '@tabler/icons-react';

export default function SettingsPage() {
  return (
    <div className="space-y-6 mx-auto container">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-default-800">Settings</h1>
        <p className="text-default-600 mt-1">Manage your account settings and preferences</p>
      </div>

      {/* Profile Settings */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <IconUser size={20} />
            <h3 className="text-lg font-semibold text-default-800">Profile</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              label="Name"
              placeholder="Enter your name"
              defaultValue="John Doe"
            />
            <Input
              label="Email"
              placeholder="Enter your email"
              defaultValue="john.doe@example.com"
              type="email"
            />
          </div>
          <Input
            label="Bio"
            placeholder="Tell us about yourself"
            defaultValue="System Administrator"
          />
          <div className="flex justify-end">
            <Button color="primary">Save Changes</Button>
          </div>
        </CardBody>
      </Card>

      {/* Notification Settings */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <IconBell size={20} />
            <h3 className="text-lg font-semibold text-default-800">Notifications</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">Email Notifications</p>
                <p className="text-sm text-default-500">Receive email notifications for important updates</p>
              </div>
              <Switch defaultSelected />
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">Push Notifications</p>
                <p className="text-sm text-default-500">Receive push notifications in your browser</p>
              </div>
              <Switch defaultSelected />
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">System Updates</p>
                <p className="text-sm text-default-500">Notify me when system updates are available</p>
              </div>
              <Switch />
            </div>
          </div>
        </CardBody>
      </Card>

      {/* Appearance Settings */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <IconPalette size={20} />
            <h3 className="text-lg font-semibold text-default-800">Appearance</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Select
              label="Theme"
              placeholder="Select theme"
              defaultSelectedKeys={["light"]}
            >
              <SelectItem key="light">Light Theme</SelectItem>
              <SelectItem key="dark">Dark Theme</SelectItem>
              <SelectItem key="auto">Follow System</SelectItem>
            </Select>
            <Select
              label="Language"
              placeholder="Select language"
              defaultSelectedKeys={["en"]}
            >
              <SelectItem key="zh-cn">简体中文</SelectItem>
              <SelectItem key="en">English</SelectItem>
              <SelectItem key="ja">日本語</SelectItem>
            </Select>
          </div>
        </CardBody>
      </Card>

      {/* Security Settings */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <IconShield size={20} />
            <h3 className="text-lg font-semibold text-default-800">Security</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">Two-Factor Authentication</p>
                <p className="text-sm text-default-500">Add an extra layer of security to your account</p>
              </div>
              <Button variant="flat" size="sm">Setup</Button>
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">Change Password</p>
                <p className="text-sm text-default-500">Update your password regularly to keep your account secure</p>
              </div>
              <Button variant="flat" size="sm">Change</Button>
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">Login Devices</p>
                <p className="text-sm text-default-500">View and manage your logged-in devices</p>
              </div>
              <Button variant="flat" size="sm">View</Button>
            </div>
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
