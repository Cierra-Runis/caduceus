'use client';

import { Button, ButtonGroup } from '@heroui/button';
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
import { match } from 'ts-pattern';

import { CreateProjectButton } from '@/components/buttons/CreateProjectButton';
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
      <CreateProjectButton ownerType='user'>
        {t('Dashboard.createProject')}
      </CreateProjectButton>
      <Table
        aria-label={t('Dashboard.yourProjects')}
        isStriped
        selectionMode='multiple'
        showDragButtons
        showSelectionCheckboxes
      >
        <TableHeader<Column>
          columns={[
            { children: t('ProjectPayload.name'), key: 'name' },
            { children: t('ProjectPayload.createdAt'), key: 'created_at' },
            { children: t('ProjectPayload.updatedAt'), key: 'updated_at' },
            { children: t('Table.actions'), key: 'actions' },
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
                    <TableCell>{item.created_at.toDateString()}</TableCell>
                  ))
                  .with('updated_at', () => (
                    <TableCell>
                      {item.updated_at.toLocaleDateString()}
                    </TableCell>
                  ))
                  .with('actions', () => (
                    <TableCell>
                      <ButtonGroup size='sm' variant='bordered'>
                        <Tooltip content={t('Table.open')}>
                          <Button
                            isIconOnly
                            startContent={<IconPlayerPlay className='w-4' />}
                          />
                        </Tooltip>
                        <Tooltip content={t('Table.download')}>
                          <Button
                            isIconOnly
                            startContent={<IconDownload className='w-4' />}
                          />
                        </Tooltip>
                        <Tooltip content={t('Table.duplicate')}>
                          <Button
                            isIconOnly
                            startContent={<IconCopy className='w-4' />}
                          />
                        </Tooltip>
                        <Tooltip content={t('Table.settings')}>
                          <Button
                            isIconOnly
                            startContent={<IconSettings className='w-4' />}
                          />
                        </Tooltip>
                      </ButtonGroup>
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
