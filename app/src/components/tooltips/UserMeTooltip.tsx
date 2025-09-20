import { Avatar } from '@heroui/avatar';
import { Tooltip } from '@heroui/tooltip';

import { useUserMe } from '@/hooks/useUserMe';

export function UserMeTooltip() {
  const { data, isLoading } = useUserMe();

  return (
    <Tooltip
      content={data?.payload?.username}
      isDisabled={isLoading || !data}
      placement='right'
    >
      <Avatar src={data?.payload.avatar_uri ?? '/icon.svg'} />
    </Tooltip>
  );
}
