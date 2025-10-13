import type { NextConfig } from 'next';

import createBundleAnalyzer from '@next/bundle-analyzer';
import createNextIntlPlugin from 'next-intl/plugin';

import { env } from '@/lib/env';

const nextConfig: NextConfig = {
  compiler: {
    removeConsole:
      env.NODE_ENV === 'production' ? { exclude: ['error'] } : false,
  },
  experimental: {
    authInterrupts: true,
  },
  reactStrictMode: true,
  typedRoutes: true,
};

const withBundleAnalyzer = createBundleAnalyzer({
  enabled: env.ANALYZE,
});

const withNextIntl = createNextIntlPlugin();

/// FIXME: https://github.com/vercel/next.js/issues/77482
export default env.ANALYZE
  ? withBundleAnalyzer(withNextIntl(nextConfig))
  : withNextIntl(nextConfig);
