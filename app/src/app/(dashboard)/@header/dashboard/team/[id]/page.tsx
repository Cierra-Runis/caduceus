import { TeamHeader } from '@/app/(dashboard)/_components/Header';

export default async function Page(props: PageProps<'/dashboard/team/[id]'>) {
  const { id } = await props.params;

  return <TeamHeader teamId={id} />;
}
