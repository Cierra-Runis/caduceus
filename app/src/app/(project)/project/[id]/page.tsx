import { HTTPError, Options } from 'ky';
import { cookies } from 'next/headers';
import { notFound, redirect } from 'next/navigation';

import { ProjectPayload } from '@/lib/api/project';
import { api } from '@/lib/request';
import { ApiResponse } from '@/lib/response';

import { ClientPage } from './_components/ClientPage';

export default async function Page(props: PageProps<'/project/[id]'>) {
  const { id } = await props.params;
  const cookieStore = await cookies();
  const options: Options = {
    headers: {
      cookie: cookieStore.toString(),
    },
  };

  try {
    const res = await api
      .get(`project/${id}`, options)
      .json<ApiResponse<ProjectPayload>>();
    return <ClientPage project={res.payload} />;
  } catch (error) {
    if (error instanceof HTTPError) {
      if (error.response.status === 404) {
        notFound();
      }
    }
    console.error(error);
    return redirect('/');
  }
}
