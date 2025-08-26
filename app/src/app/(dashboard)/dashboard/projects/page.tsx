export default function ProjectsPage() {
  return (
    <div className="space-y-6 mx-auto container">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-default-800">Projects</h1>
        <p className="text-default-600 mt-1">Manage your projects and tasks</p>
      </div>
      {/* Content */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {/* Example Project Card */}
        <div className="border border-divider rounded-medium p-4 hover:shadow-lg transition-shadow">
          <h2 className="text-xl font-semibold text-default-800">Project Alpha</h2>
          <p className="text-default-600 mt-2">A brief description of Project Alpha.</p>
          <div className="mt-4 flex justify-between items-center">
            <span className="text-sm text-default-500">Due: 2024-12-31</span>
            <span className="text-sm text-default-500">Status: In Progress</span>
          </div>
        </div>
        {/* Add more project cards as needed */}
      </div>
    </div>
  );
}