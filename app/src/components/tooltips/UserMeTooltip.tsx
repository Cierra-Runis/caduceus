import { Avatar } from '@heroui/avatar';
import { Tooltip } from '@heroui/tooltip';

import { useUserMe } from '@/hooks/useUserMe';

export function UserMeTooltip() {
  const { data, isLoading } = useUserMe();

  return (
    <Tooltip
      content={data?.payload?.username}
      isDisabled={isLoading}
      placement='right'
    >
      <Avatar src='https://i.pravatar.cc?img=1' />
    </Tooltip>
  );
}
