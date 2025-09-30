import type { NextConfig } from 'next';

import createBundleAnalyzer from '@next/bundle-analyzer';
import createNextIntlPlugin from 'next-intl/plugin';

const nextConfig: NextConfig = {
  compiler: {
    removeConsole:
      process.env.NODE_ENV === 'production' ? { exclude: ['error'] } : false,
  },
  reactStrictMode: true,
  typedRoutes: true,
};

const withBundleAnalyzer = createBundleAnalyzer({
  enabled: process.env.ANALYZE === 'true',
});

const withNextIntl = createNextIntlPlugin();

/// FIXME: https://github.com/vercel/next.js/issues/77482
export default process.env.ANALYZE === 'true'
  ? withBundleAnalyzer(withNextIntl(nextConfig))
  : withNextIntl(nextConfig);
