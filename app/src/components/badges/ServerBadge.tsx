"use client";

import { useServerStatus } from "@/hooks/useServerStatus";
import { Badge, BadgeProps } from "@heroui/badge";

export function ServerBadge(props: BadgeProps) {
  const { color } = useServerStatus();

  return <Badge color={color} content="" placement="bottom-right" shape="circle" {...props} />
}