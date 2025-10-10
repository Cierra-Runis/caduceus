import { createEnv } from '@t3-oss/env-nextjs';
import * as z from 'zod';

export const env = createEnv({
  client: {
    NEXT_PUBLIC_API_URL: z.url(),
    NEXT_PUBLIC_WS_URL: z.url(),
  },
  experimental__runtimeEnv: {
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL,
    NEXT_PUBLIC_WS_URL: process.env.NEXT_PUBLIC_WS_URL,
  },
  server: {
    ANALYZE: z
      .string()
      .optional()
      .transform((val) => val === 'true'),
    JWT_SECRET: z.string().nonempty(),
    NODE_ENV: z.enum(['development', 'test', 'production']),
  },
});
