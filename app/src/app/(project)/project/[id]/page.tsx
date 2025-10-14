import { HTTPError, Options } from 'ky';
import { cookies } from 'next/headers';
import { forbidden, notFound, redirect, unauthorized } from 'next/navigation';
import { match } from 'ts-pattern';

import { CreateProjectResponseSchema } from '@/lib/api/project';
import { api } from '@/lib/request';

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
    const res = await api.get(`project/${id}`, options).json();
    const parsed = CreateProjectResponseSchema.parse(res);
    return <ClientPage project={parsed.payload} />;
  } catch (error) {
    if (error instanceof HTTPError) {
      return match(error.response.status)
        .with(401, () => unauthorized())
        .with(403, () => forbidden())
        .with(404, () => notFound())
        .otherwise(() => redirect('/'));
    }
    console.error(error);
    return redirect('/');
  }
}
