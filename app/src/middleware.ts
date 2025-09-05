import { NextRequest, NextResponse } from 'next/server';

import { decrypt } from '@/lib/session';

// Specify protected and public routes
const protectedRoutes = ['/dashboard'];
const publicRoutes = ['/login', '/register'];

// Middleware to handle authentication and route protection
//
// There are four types of routes:
//
// - Root route: show dashboard if authenticated, otherwise show home page
// - Protected routes: require authentication (e.g., /dashboard)
// - Public routes: should not be accessible after authenticated (e.g., /login, /register)
// - Unrestricted routes: accessible to all users (e.g., /about)
//
// The middleware checks the user's authentication status based on a JWT stored in cookies
// and redirects them accordingly.
export default async function middleware(req: NextRequest) {
  const path = req.nextUrl.pathname;

  // Decrypt the jwt from the cookie
  const payload = await decrypt(req.cookies.get('token')?.value);
  const isAuthenticated = !!payload?.sub;

  // If the route is root, redirect based on authentication status
  if (path === '/') {
    if (!isAuthenticated) {
      return NextResponse.redirect(new URL('/home', req.url));
    }
    return NextResponse.next();
  }

  // Check if the route is protected
  const isProtectedRoute = protectedRoutes.some((r) => path.startsWith(r));
  // Redirect to /login if the user is not authenticated
  if (isProtectedRoute && !isAuthenticated) {
    return NextResponse.redirect(new URL('/login', req.nextUrl));
  }

  // Check if the route is public
  const isPublicRoute = publicRoutes.includes(path);
  // Redirect to / if the user is authenticated
  if (isPublicRoute && isAuthenticated) {
    return NextResponse.redirect(new URL('/', req.nextUrl));
  }

  // For unrestricted routes or if no redirection is needed, continue the request
  return NextResponse.next();
}
