'use client';

import { useServerStatus } from '@/hooks/useServerStatus';
import { Badge, BadgeProps } from '@heroui/badge';

type ServerBadgeProps = Omit<BadgeProps, 'children'> & {
  children?: BadgeProps['children']; // make children optional for this wrapper
};

export function ServerBadge({ children = '', ...props }: ServerBadgeProps) {
  const { color } = useServerStatus();

  return (
    <Badge
      color={color}
      content=''
      placement='bottom-right'
      shape='circle'
      {...props}
    >
      {children}
    </Badge>
  );
}
