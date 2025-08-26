// This page renders when a route like `/unknown.txt` is requested.

import RootLayout from "./(home)/layout";

// In this case, the layout at `app/layout.tsx` receives.
export default function GlobalNotFound() {
  return (
    <RootLayout>
      <div className="flex h-full items-center justify-center">
        <div className="flex items-center flex-col text-center">
          <div className="text-4xl font-bold">404</div>
          <div className="mt-4 text-lg">This page could not be found.</div>
        </div>
      </div>;
    </RootLayout>
  )
}