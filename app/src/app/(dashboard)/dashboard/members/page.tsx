export default function MembersPage() {
  return (
    <div className="space-y-6 mx-auto container">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-default-800">Members</h1>
        <p className="text-default-600 mt-1">Manage your team members and roles</p>
      </div>
      {/* Content */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* Example Member Card */}
        <div className="border border-divider rounded-medium p-4 hover:shadow-lg transition-shadow">
          <h2 className="text-xl font-semibold text-default-800">John Doe</h2>
          <p className="text-default-600 mt-2">Role: Project Manager</p>
          <div className="mt-4 flex justify-between items-center">
            <span className="text-sm text-default-500">Email: example@example.com</span>
            <span className="text-sm text-default-500">Status: Active</span>
          </div>
        </div>
        {/* Add more member cards as needed */}
      </div>
    </div>
  );
}