import Link from 'next/link'

export default function NextraNavbar() {
  return (
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
  )
}
