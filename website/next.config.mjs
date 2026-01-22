import nextra from 'nextra'

const withNextra = nextra({
  defaultShowCopyCode: true,
  latex: true,
})

/**
 * @type {import('next').NextConfig}
 */
const nextConfig = {
  // Remove output: 'export' for now to test if it builds
  images: {
    unoptimized: true,
  },
  pageExtensions: ['js', 'jsx', 'ts', 'tsx', 'md', 'mdx'],
}

export default withNextra(nextConfig)
