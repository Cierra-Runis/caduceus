import { NextRequest, NextResponse } from 'next/server';

import { decrypt } from '@/lib/session';

// Specify protected and public routes
const protectedRoutes = ['/dashboard'];
const publicRoutes = ['/login', '/register'];

// Middleware to handle authentication and route protection
//
// There are four types of routes:
// - Root route: show dashboard if authenticated, otherwise show landing page
// - Protected routes: require authentication (e.g., /)
// - Public routes: should not be accessible after authenticated (e.g., /login, /register)
// - Unrestricted routes: accessible to all users (e.g., /about)
//
// The middleware checks the user's authentication status based on a JWT stored in cookies
// and redirects them accordingly.
export default async function middleware(req: NextRequest) {
  const path = req.nextUrl.pathname;

  console.log('[Middleware] Path:', path);

  // Decrypt the jwt from the cookie
  const payload = await decrypt(req.cookies.get('token')?.value);
  const isAuthenticated = !!payload?.sub;

  console.log('[Middleware] Authenticated:', isAuthenticated);
  console.log('[Middleware] Payload:', payload);

  // If the route is root, redirect based on authentication status
  if (path === '/') {
    if (!isAuthenticated) {
      console.log('Redirecting to /(home)/home/page.tsx');
      return NextResponse.redirect(new URL('/home', req.url));
    }
    console.log('Keep going to /(dashboard)/page.tsx');
    return NextResponse.next();
  }

  // Check if the route is protected
  const isProtectedRoute = protectedRoutes.some((r) => path.startsWith(r));
  // Redirect to /login if the user is not authenticated
  if (isProtectedRoute && !isAuthenticated) {
    console.log('Redirecting to /(home)/login/page.tsx');
    return NextResponse.redirect(new URL('/login', req.nextUrl));
  }

  // Check if the route is public
  const isPublicRoute = publicRoutes.includes(path);
  // Redirect to / if the user is authenticated
  if (isPublicRoute && isAuthenticated) {
    console.log('Redirecting to /');
    return NextResponse.redirect(new URL('/', req.nextUrl));
  }

  // For unrestricted routes or if no redirection is needed, continue the request
  return NextResponse.next();
}
