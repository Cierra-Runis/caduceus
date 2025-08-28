import type { NextConfig } from 'next';

import createNextIntlPlugin from 'next-intl/plugin';

const nextConfig: NextConfig = {
  reactStrictMode: true,
  async rewrites() {
    // Proxy the front-end /api/*request to the local Go service (development environment)
    return [
      {
        destination: 'http://localhost:8080/api/:path*',
        source: '/api/:path*',
      },
    ];
  },
};

const withNextIntl = createNextIntlPlugin();

export default withNextIntl(nextConfig);
