export default function MembersPage() {
  return (
    <div className='container mx-auto space-y-6'>
      {/* Header */}
      <div>
        <h1 className='text-default-800 text-3xl font-bold'>Members</h1>
        <p className='text-default-600 mt-1'>
          Manage your team members and roles
        </p>
      </div>
      {/* Content */}
      <div className='grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3'>
        {/* Example Member Card */}
        <div className='border-divider rounded-medium border p-4 transition-shadow hover:shadow-lg'>
          <h2 className='text-default-800 text-xl font-semibold'>John Doe</h2>
          <p className='text-default-600 mt-2'>Role: Project Manager</p>
          <div className='mt-4 flex items-center justify-between'>
            <span className='text-default-500 text-sm'>
              Email: example@example.com
            </span>
            <span className='text-default-500 text-sm'>Status: Active</span>
          </div>
        </div>
        {/* Add more member cards as needed */}
      </div>
    </div>
  );
}
