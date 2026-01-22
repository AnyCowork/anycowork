import { Layout } from 'nextra-theme-docs'
import { getPageMap } from 'nextra/page-map'

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
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <img src="/logo.svg" alt="AnyCowork" style={{ height: '28px', width: '28px' }} />
          <span style={{ fontWeight: 700, fontSize: '1.2rem' }}>AnyCowork</span>
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
