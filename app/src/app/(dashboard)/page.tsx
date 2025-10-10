'use client';

import { Button } from '@heroui/button';
import { Spinner } from '@heroui/spinner';
import {
  Table,
  TableBody,
  TableCell,
  TableColumn,
  TableColumnProps,
  TableHeader,
  TableRow,
} from '@heroui/table';
import { Tooltip } from '@heroui/tooltip';
import {
  IconCopy,
  IconDownload,
  IconPlayerPlay,
  IconSettings,
} from '@tabler/icons-react';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';
import { match } from 'ts-pattern';

import { CreateProjectButton } from '@/components/buttons/CreateProjectButton';
import { UpdateProjectButton } from '@/components/buttons/UpdateProjectButton';
import { useUserProject } from '@/hooks/api/user/project';
import { Project } from '@/lib/types/project';

type Column = {
  key: 'actions' | ({} & keyof Project);
} & TableColumnProps<unknown>;

export default function Dashboard() {
  const t = useTranslations();
  const { data, error, isLoading } = useUserProject();

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
    <main className='flex h-full items-center justify-center'>
      <Table
        aria-label={t('Dashboard.yourProjects')}
        topContent={
          <div className='flex items-center justify-between'>
            <div>
              <CreateProjectButton ownerType='user' variant='bordered'>
                {t('Dashboard.createProject')}
              </CreateProjectButton>
            </div>
            <div>
              <Button isIconOnly variant='ghost'>
                <IconSettings className='w-4' />
              </Button>
            </div>
          </div>
        }
        topContentPlacement='outside'
      >
        <TableHeader<Column>
          columns={[
            { children: t('ProjectPayload.name'), key: 'name' },
            { children: t('ProjectPayload.createdAt'), key: 'created_at' },
            { children: t('ProjectPayload.updatedAt'), key: 'updated_at' },
            { align: 'center', children: t('Table.actions'), key: 'actions' },
          ]}
        >
          {({ children, key, ...props }) => (
            <TableColumn key={key} {...props}>
              {children}
            </TableColumn>
          )}
        </TableHeader>
        <TableBody
          emptyContent={t('Dashboard.noProjects')}
          items={data.payload}
        >
          {(item) => (
            <TableRow key={item.id}>
              {(columnKey) => {
                return match(columnKey as Column['key'])
                  .with('id', () => <TableCell>{item.id}</TableCell>)
                  .with('owner_id', () => (
                    <TableCell>{item.owner_id}</TableCell>
                  ))
                  .with('owner_type', () => (
                    <TableCell>{item.owner_type}</TableCell>
                  ))
                  .with('creator_id', () => (
                    <TableCell>{item.creator_id}</TableCell>
                  ))
                  .with('name', () => <TableCell>{item.name}</TableCell>)
                  .with('created_at', () => (
                    <TableCell>
                      {item.created_at.toLocaleDateString()}
                    </TableCell>
                  ))
                  .with('updated_at', () => (
                    <TableCell>
                      {item.updated_at.toLocaleDateString()}
                    </TableCell>
                  ))
                  .with('actions', () => (
                    <TableCell
                      className={`flex items-center justify-center gap-1`}
                    >
                      <Tooltip content={t('Table.open')}>
                        <Button
                          as={NextLink}
                          href={`/project/${item.id}`}
                          isIconOnly
                          size='sm'
                          startContent={<IconPlayerPlay className='w-4' />}
                          variant='bordered'
                        />
                      </Tooltip>
                      <Tooltip content={t('Table.download')}>
                        <Button
                          isIconOnly
                          size='sm'
                          startContent={<IconDownload className='w-4' />}
                          // TODO: Implement
                          variant='bordered'
                        />
                      </Tooltip>
                      <Tooltip content={t('Table.duplicate')}>
                        <Button
                          isIconOnly
                          size='sm'
                          startContent={<IconCopy className='w-4' />}
                          // TODO: Implement
                          variant='bordered'
                        />
                      </Tooltip>
                      <Tooltip content={t('Table.settings')}>
                        <UpdateProjectButton
                          isIconOnly
                          project={item}
                          size='sm'
                          startContent={<IconSettings className='w-4' />}
                          variant='bordered'
                        />
                      </Tooltip>
                    </TableCell>
                  ))
                  .exhaustive();
              }}
            </TableRow>
          )}
        </TableBody>
      </Table>
    </main>
  );
}
