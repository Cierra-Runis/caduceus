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
import { HTTPError } from 'ky';
import { useTranslations } from 'next-intl';
import useSWR from 'swr';
import { match } from 'ts-pattern';
import * as z from 'zod';

import { CreateProjectButton } from '@/components/buttons/CreateProjectButton';
import { api } from '@/lib/request';
import { Project, ProjectSchema } from '@/lib/types/project';

type Column = {
  key: 'actions' | ({} & keyof Project);
} & TableColumnProps<unknown>;

type UserProjectResponse = z.infer<typeof UserProjectResponseSchema>;
const UserProjectResponseSchema = z.object({
  payload: z.array(ProjectSchema),
});

export default function Dashboard() {
  const t = useTranslations();
  const { data, error, isLoading } = useSWR<
    UserProjectResponse,
    HTTPError,
    string
  >('user/projects', async (key) =>
    UserProjectResponseSchema.parse(await api.get(key).json()),
  );

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
            { children: t('ProjectPayload.id'), key: 'id' },
            { children: t('ProjectPayload.ownerId'), key: 'owner_id' },
            { children: t('ProjectPayload.ownerType'), key: 'owner_type' },
            { children: 'Actions', key: 'actions' },
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
                      {new Date(item.updated_at).toLocaleDateString()}
                    </TableCell>
                  ))
                  .with('actions', () => (
                    <TableCell>
                      <Button />
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
