import { ClientPage } from './_components/ClientPage';

export default async function TeamPage(
  props: PageProps<'/dashboard/team/[id]'>,
) {
  const { id } = await props.params;

  return <ClientPage id={id} />;
}
