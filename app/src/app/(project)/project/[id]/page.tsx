import { ClientPage } from './_components/ClientPage';

export default async function Page(props: PageProps<'/project/[id]'>) {
  const { id } = await props.params;

  return <ClientPage projectId={id} />;
}
