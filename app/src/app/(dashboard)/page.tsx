'use client';

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
import { useTranslations } from 'next-intl';
import useSWR from 'swr';

import { CreateProjectButton } from '@/components/buttons/CreateProjectButton';
import { ProjectPayload } from '@/lib/api/project';
import { api } from '@/lib/request';
import { ApiResponse, ErrorResponse } from '@/lib/response';

type Column = {
  key: keyof ProjectPayload;
} & TableColumnProps<unknown>;

type UserProjectResponse = ApiResponse<ProjectPayload[]>;

export default function Dashboard() {
  const t = useTranslations();
  const { data, error, isLoading } = useSWR<
    UserProjectResponse,
    ErrorResponse,
    string
  >('user/projects', (key) => api.get(key).json());

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
                const value = item[columnKey as keyof ProjectPayload];
                return <TableCell>{value.toLocaleString()}</TableCell>;
              }}
            </TableRow>
          )}
        </TableBody>
      </Table>
    </main>
  );
}
