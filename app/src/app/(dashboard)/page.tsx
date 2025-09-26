'use client';

import { Listbox, ListboxItem } from '@heroui/listbox';
import { Spinner } from '@heroui/spinner';
import NextLink from 'next/link';
import useSWR from 'swr';

import { CreateProjectButton } from '@/components/buttons/CreateProjectButton';
import { ProjectPayload } from '@/lib/api/project';
import { api } from '@/lib/request';
import { ApiResponse, ErrorResponse } from '@/lib/response';

type UserProjectResponse = ApiResponse<ProjectPayload[]>;

export default function Dashboard() {
  const { data, error, isLoading } = useSWR<
    UserProjectResponse,
    ErrorResponse,
    string
  >('/api/user/projects', (key) => api.get(key).json());

  if (isLoading)
    return (
      <div className='flex h-full items-center justify-center'>
        <Spinner />
      </div>
    );
  if (error || !data)
    return (
      <div className='flex h-full items-center justify-center'>
        {error?.message || 'Failed to load projects.'}
      </div>
    );

  return (
    <main className='flex h-full items-center justify-center'>
      <CreateProjectButton ownerType='user'>Create Project</CreateProjectButton>
      <Listbox className='w-72' label='Your Projects'>
        {data.payload.map((project) => (
          <ListboxItem
            as={NextLink}
            href={`/project/${project.id}`}
            key={project.id}
          >
            {project.name}
          </ListboxItem>
        ))}
      </Listbox>
    </main>
  );
}
