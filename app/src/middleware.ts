import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';

import { decrypt } from '@/lib/session';

// Specify protected and public routes
const protectedRoutes = ['/dashboard'];
const publicRoutes = ['/login', '/register', '/'];

export default async function middleware(req: NextRequest) {
  // Check if the current route is protected or public
  const path = req.nextUrl.pathname;
  const isProtectedRoute = protectedRoutes.includes(path);
  const isPublicRoute = publicRoutes.includes(path);

  // Decrypt the jwt from the cookie
  const cookie = await cookies();
  const token = cookie.get('token')?.value;
  const payload = await decrypt(token);

  // Redirect to /login if the user is not authenticated
  if (isProtectedRoute && !payload?.sub) {
    return NextResponse.redirect(new URL('/login', req.nextUrl));
  }

  // Redirect to /dashboard if the user is authenticated
  if (
    isPublicRoute &&
    payload?.sub &&
    !req.nextUrl.pathname.startsWith('/dashboard')
  ) {
    return NextResponse.redirect(new URL('/dashboard', req.nextUrl));
  }

  return NextResponse.next();
}

// Routes Middleware should not run on
export const config = {
  matcher: ['/((?!api|_next/static|_next/image|.*\\.png$).*)'],
};
