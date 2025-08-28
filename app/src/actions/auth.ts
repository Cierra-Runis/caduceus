'use server';

import { redirect } from 'next/navigation';

import { deleteJwt } from '@/lib/session';

export async function logout() {
  await deleteJwt();
  redirect('/login');
}
