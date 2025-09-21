'use client';

import { Listbox, ListboxItem } from '@heroui/listbox';
import useSWR from 'swr';

import { CreateProjectButton } from '@/components/buttons/CreateProjectButton';
import { ProjectPayload } from '@/lib/api/project';
import { api } from '@/lib/request';
import { ApiResponse, ErrorResponse } from '@/lib/response';

type UserProjectResponse = ApiResponse<ProjectPayload[]>;

export default function Dashboard() {
  const { data } = useSWR<UserProjectResponse, ErrorResponse, string>(
    '/api/user/projects',
    (key) => api.get(key).json(),
  );

  if (!data) {
    return <div>Loading...</div>;
  }

  return (
    <main className='flex h-full items-center justify-center'>
      <CreateProjectButton ownerType='user'>Create Project</CreateProjectButton>
      <Listbox className='w-72' label='Your Projects'>
        {data.payload.map((project) => (
          <ListboxItem key={project.id}>{project.name}</ListboxItem>
        ))}
      </Listbox>
    </main>
  );
}
