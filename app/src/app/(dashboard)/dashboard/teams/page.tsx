export default function TeamsPage() {
  return (
    <div className='container mx-auto space-y-6'>
      {/* Header */}
      <div>
        <h1 className='text-default-800 text-3xl font-bold'>Teams</h1>
        <p className='text-default-600 mt-1'>
          Manage your teams and collaborations
        </p>
      </div>
      {/* Content */}
      <div className='grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3'>
        {/* Example Team Card */}
        <div className='border-divider rounded-medium border p-4 transition-shadow hover:shadow-lg'>
          <h2 className='text-default-800 text-xl font-semibold'>Team A</h2>
          <p className='text-default-600 mt-2'>
            A brief description of Team A.
          </p>
          <div className='mt-4 flex items-center justify-between'>
            <span className='text-default-500 text-sm'>Members: 10</span>
            <span className='text-default-500 text-sm'>Projects: 5</span>
          </div>
        </div>
        {/* Add more team cards as needed */}
      </div>
    </div>
  );
}
