export default function ProjectsPage() {
  return (
    <div className='container mx-auto space-y-6'>
      {/* Header */}
      <div>
        <h1 className='text-default-800 text-3xl font-bold'>Projects</h1>
        <p className='text-default-600 mt-1'>Manage your projects and tasks</p>
      </div>
      {/* Content */}
      <div className='grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3'>
        {/* Example Project Card */}
        <div className='border-divider rounded-medium border p-4 transition-shadow hover:shadow-lg'>
          <h2 className='text-default-800 text-xl font-semibold'>
            Project Alpha
          </h2>
          <p className='text-default-600 mt-2'>
            A brief description of Project Alpha.
          </p>
          <div className='mt-4 flex items-center justify-between'>
            <span className='text-default-500 text-sm'>Due: 2024-12-31</span>
            <span className='text-default-500 text-sm'>
              Status: In Progress
            </span>
          </div>
        </div>
        {/* Add more project cards as needed */}
      </div>
    </div>
  );
}
