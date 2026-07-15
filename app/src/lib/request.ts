import ky from 'ky';

import { env } from '@/lib/env';

export const api = ky.extend({
  credentials: 'include',
  prefix: env.NEXT_PUBLIC_API_URL,
});
