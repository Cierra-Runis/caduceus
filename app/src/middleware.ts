import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';

import { decrypt } from '@/lib/session';

// 1. Specify protected and public routes
const protectedRoutes = ['/dashboard'];
const publicRoutes = ['/login', '/register', '/'];
const logoutRoute = '/logout';

export default async function middleware(req: NextRequest) {
  // 2. Check if the current route is protected or public
  const path = req.nextUrl.pathname;
  const isProtectedRoute = protectedRoutes.includes(path);
  const isPublicRoute = publicRoutes.includes(path);
  const isLogoutRoute = path === logoutRoute;

  // 3. Handle logout
  if (isLogoutRoute) {
    const cookie = await cookies();
    cookie.delete('jwt');
    return NextResponse.redirect(new URL('/', req.nextUrl));
  }

  // 4. Decrypt the jwt from the cookie
  const cookie = await cookies();
  const jwt = cookie.get('jwt')?.value;
  const payload = await decrypt(jwt);

  // 5. Redirect to /login if the user is not authenticated
  if (isProtectedRoute && !payload?.username) {
    return NextResponse.redirect(new URL('/login', req.nextUrl));
  }

  // 6. Redirect to /dashboard if the user is authenticated
  if (
    isPublicRoute &&
    payload?.username &&
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
