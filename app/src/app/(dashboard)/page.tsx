'use client';

import { CopyIcon, PlayIcon, SettingsIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

import { CreateProjectButton } from '@/components/buttons/CreateProjectButton';
import { DownloadProjectButton } from '@/components/buttons/DownloadProjectButton';
import { UpdateProjectButton } from '@/components/buttons/UpdateProjectButton';
import { Button } from '@/components/ui/button';
import { Spinner } from '@/components/ui/spinner';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useUserMe } from '@/hooks/api/user/me';
import { useUserProject } from '@/hooks/api/user/project';

export default function Dashboard() {
  const t = useTranslations();
  const { data, error, isLoading } = useUserProject();
  const { data: user } = useUserMe();

  if (isLoading)
    return (
      <div className='flex h-full items-center justify-center'>
        <Spinner />
      </div>
    );
  if (error || !data)
    return (
      <div className='flex h-full items-center justify-center'>
        {error?.message || t('Dashboard.failedToLoad')}
      </div>
    );

  return (
    <main className='flex h-full flex-col gap-4 p-6'>
      <div className='flex items-center justify-between'>
        <h1 className='font-heading text-lg font-medium'>
          {t('Dashboard.yourProjects')}
        </h1>
        <CreateProjectButton ownerId={user?.payload.id ?? ''} ownerType='user'>
          {t('Dashboard.createProject')}
        </CreateProjectButton>
      </div>

      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{t('ProjectPayload.name')}</TableHead>
            <TableHead>{t('ProjectPayload.createdAt')}</TableHead>
            <TableHead>{t('ProjectPayload.updatedAt')}</TableHead>
            <TableHead className='text-center'>{t('Table.actions')}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {data.payload.length === 0 ? (
            <TableRow>
              <TableCell
                className='h-24 text-center text-muted-foreground'
                colSpan={4}
              >
                {t('Dashboard.noProjects')}
              </TableCell>
            </TableRow>
          ) : (
            data.payload.map((item) => (
              <TableRow key={item.id}>
                <TableCell className='font-medium'>{item.name}</TableCell>
                <TableCell>{item.created_at.toLocaleDateString()}</TableCell>
                <TableCell>{item.updated_at.toLocaleDateString()}</TableCell>
                <TableCell>
                  <div className='flex items-center justify-center gap-1'>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button asChild size='icon-sm' variant='outline'>
                          <NextLink href={`/project/${item.id}`}>
                            <PlayIcon />
                          </NextLink>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>{t('Table.open')}</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <DownloadProjectButton
                          project={item}
                          size='icon-sm'
                          variant='outline'
                        />
                      </TooltipTrigger>
                      <TooltipContent>{t('Table.download')}</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        {/* TODO: Implement duplicate */}
                        <Button size='icon-sm' variant='outline'>
                          <CopyIcon />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>{t('Table.duplicate')}</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <UpdateProjectButton
                          project={item}
                          size='icon-sm'
                          variant='outline'
                        >
                          <SettingsIcon />
                        </UpdateProjectButton>
                      </TooltipTrigger>
                      <TooltipContent>{t('Table.settings')}</TooltipContent>
                    </Tooltip>
                  </div>
                </TableCell>
              </TableRow>
            ))
          )}
        </TableBody>
      </Table>
    </main>
  );
}
