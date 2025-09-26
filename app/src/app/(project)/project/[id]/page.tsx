import ky, { HTTPError } from 'ky';
import { cookies } from 'next/headers';
import { notFound, redirect } from 'next/navigation';

import { ProjectPayload } from '@/lib/api/project';
import { ApiResponse } from '@/lib/response';

import { ClientPage } from './_components/ClientPage';

export default async function Page(props: PageProps<'/project/[id]'>) {
  const { id } = await props.params;
  const cookieStore = await cookies();

  try {
    const res = await ky
      .get(`http://localhost:8080/api/project/${id}`, {
        headers: {
          cookie: cookieStore.toString(),
        },
      })
      .json<ApiResponse<ProjectPayload>>();

    return <ClientPage project={res.payload} />;
  } catch (error) {
    if (error instanceof HTTPError) {
      if (error.response.status === 404) {
        notFound();
      }
    }
    return redirect('/');
  }
}
