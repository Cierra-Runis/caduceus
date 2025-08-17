import { ServerBadge } from "@/components/badges/ServerBadge";
import { Avatar } from "@heroui/avatar";

export default function Home() {
  return (
    <div className="flex items-center justify-center h-screen">
      <ServerBadge>
        <Avatar src="https://avatars.githubusercontent.com/u/29329988" />
      </ServerBadge>
    </div>
  );
}
