import { Layout } from 'nextra-theme-docs'
import { getPageMap } from 'nextra/page-map'
import Link from 'next/link'

export default async function BlogLayout({
  children,
}: {
  children: React.ReactNode
}) {
  const pageMap = await getPageMap('/blog')

  return (
    <Layout
      pageMap={pageMap}
      docsRepositoryBase="https://github.com/anycowork/anycowork/tree/main/website"
      navbar={(
        <div style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          width: '100%',
          gap: '1rem'
        }}>
          <Link href="/" style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            textDecoration: 'none',
            color: 'inherit'
          }}>
            <img src="/logo.svg" alt="AnyCowork" style={{ height: '28px', width: '28px' }} />
            <span style={{ fontWeight: 700, fontSize: '1.2rem' }}>AnyCowork</span>
          </Link>

          <div style={{
            display: 'flex',
            alignItems: 'center',
            gap: '1.5rem'
          }}>
            <Link href="/features" style={{ textDecoration: 'none', color: 'inherit' }}>
              Features
            </Link>
            <Link href="/docs" style={{ textDecoration: 'none', color: 'inherit' }}>
              Docs
            </Link>
            <Link href="/blog" style={{ textDecoration: 'none', color: 'inherit' }}>
              Blog
            </Link>
            <a
              href="https://github.com/AnyCowork/AnyCowork"
              target="_blank"
              rel="noopener noreferrer"
              style={{ textDecoration: 'none', color: 'inherit' }}
            >
              GitHub
            </a>
          </div>
        </div>
      )}
      sidebar={{
        defaultMenuCollapseLevel: 1,
        toggleButton: true,
      }}
      footer={(
        <span>
          MIT {new Date().getFullYear()} © <a href="https://anycowork.com" target="_blank" rel="noopener noreferrer">AnyCowork</a>
        </span>
      )}
    >
      {children}
    </Layout>
  )
}
