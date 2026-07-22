import { UserHeader } from '@/app/(dashboard)/_components/Header';

// Hard-load fallback for any route without an explicit header page in this
// slot. New routes should still add their own header page: on *soft*
// navigation an unmatched slot keeps its previous content instead of falling
// back here.
export default function Default() {
  return <UserHeader />;
}
