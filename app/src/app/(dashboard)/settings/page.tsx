import { Button } from '@heroui/button';
import { Card, CardBody, CardHeader } from '@heroui/card';
import { Divider } from '@heroui/divider';
import { Input } from '@heroui/input';
import { Select, SelectItem } from '@heroui/select';
import { Switch } from '@heroui/switch';
import { IconBell, IconPalette, IconShield, IconUser } from '@tabler/icons-react';

export default function SettingsPage() {
  return (
    <div className="space-y-6 max-w-4xl">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-default-800">设置</h1>
        <p className="text-default-600 mt-1">管理您的账户设置和偏好</p>
      </div>

      {/* Profile Settings */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <IconUser size={20} />
            <h3 className="text-lg font-semibold text-default-800">个人资料</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Input
              label="姓名"
              placeholder="输入您的姓名"
              defaultValue="张三"
            />
            <Input
              label="邮箱"
              placeholder="输入您的邮箱"
              defaultValue="zhangsan@example.com"
              type="email"
            />
          </div>
          <Input
            label="个人简介"
            placeholder="简单介绍一下您自己"
            defaultValue="系统管理员"
          />
          <div className="flex justify-end">
            <Button color="primary">保存更改</Button>
          </div>
        </CardBody>
      </Card>

      {/* Notification Settings */}
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center gap-2">
            <IconBell size={20} />
            <h3 className="text-lg font-semibold text-default-800">通知设置</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">邮件通知</p>
                <p className="text-sm text-default-500">接收重要更新的邮件通知</p>
              </div>
              <Switch defaultSelected />
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">推送通知</p>
                <p className="text-sm text-default-500">在浏览器中接收推送通知</p>
              </div>
              <Switch defaultSelected />
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">系统更新</p>
                <p className="text-sm text-default-500">当系统有更新时通知我</p>
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
            <h3 className="text-lg font-semibold text-default-800">外观设置</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <Select
              label="主题"
              placeholder="选择主题"
              defaultSelectedKeys={["light"]}
            >
              <SelectItem key="light">浅色主题</SelectItem>
              <SelectItem key="dark">深色主题</SelectItem>
              <SelectItem key="auto">跟随系统</SelectItem>
            </Select>
            <Select
              label="语言"
              placeholder="选择语言"
              defaultSelectedKeys={["zh-cn"]}
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
            <h3 className="text-lg font-semibold text-default-800">安全设置</h3>
          </div>
        </CardHeader>
        <CardBody className="space-y-4">
          <div className="space-y-4">
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">两步验证</p>
                <p className="text-sm text-default-500">为您的账户添加额外的安全保护</p>
              </div>
              <Button variant="flat" size="sm">设置</Button>
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">修改密码</p>
                <p className="text-sm text-default-500">定期更新密码以保护账户安全</p>
              </div>
              <Button variant="flat" size="sm">修改</Button>
            </div>
            <Divider />
            <div className="flex justify-between items-center">
              <div>
                <p className="font-medium text-default-800">登录设备</p>
                <p className="text-sm text-default-500">查看和管理已登录的设备</p>
              </div>
              <Button variant="flat" size="sm">查看</Button>
            </div>
          </div>
        </CardBody>
      </Card>
    </div>
  );
}
