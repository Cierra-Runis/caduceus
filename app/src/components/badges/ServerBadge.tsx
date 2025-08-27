'use client';

import { Badge, BadgeProps } from '@heroui/badge';

import { useServerStatus } from '@/hooks/useServerStatus';

type ServerBadgeProps = {
  children?: BadgeProps['children']; // make children optional for this wrapper
} & Omit<BadgeProps, 'children'>;

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
