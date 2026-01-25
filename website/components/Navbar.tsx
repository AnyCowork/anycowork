import { useState } from 'react'
import { Menu, X } from 'lucide-react'
import ThemeToggle from './ThemeToggle'

// Map Next.js Link to standard a tag or wrapper
const Link = ({ href, className, children, onClick }: { href: string; className?: string; children: React.ReactNode; onClick?: () => void }) => {
    return (
        <a href={href} className={className} onClick={onClick}>
            {children}
        </a>
    );
};

export default function Navbar() {
    const [isMenuOpen, setIsMenuOpen] = useState(false)

    return (
        <nav className="navbar">
            <div className="navbar-container">
                <Link href="/" className="navbar-logo">
                    <img src="/logo.svg" alt="AnyCowork" className="navbar-logo-img" />
                    <span className="navbar-logo-text">AnyCowork</span>
                </Link>

                {/* Desktop Menu */}
                <div className="navbar-menu">
                    <Link href="/docs" className="navbar-link">Docs</Link>
                    <Link href="/docs/blog" className="navbar-link">Blog</Link>
                    <a
                        href="https://github.com/AnyCowork/AnyCowork"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="navbar-link"
                    >
                        GitHub
                    </a>
                    <ThemeToggle />
                    <Link href="/docs/getting-started" className="navbar-cta">
                        Get Started
                    </Link>
                </div>

                {/* Mobile Menu Button */}
                <button
                    className="navbar-mobile-toggle"
                    onClick={() => setIsMenuOpen(!isMenuOpen)}
                    aria-label="Toggle menu"
                >
                    {isMenuOpen ? <X size={24} /> : <Menu size={24} />}
                </button>
            </div>

            {/* Mobile Menu */}
            {isMenuOpen && (
                <div className="navbar-mobile-menu">
                    <Link href="/docs" className="navbar-mobile-link" onClick={() => setIsMenuOpen(false)}>
                        Docs
                    </Link>
                    <Link href="/docs/blog" className="navbar-mobile-link" onClick={() => setIsMenuOpen(false)}>
                        Blog
                    </Link>
                    <a
                        href="https://github.com/AnyCowork/AnyCowork"
                        target="_blank"
                        rel="noopener noreferrer"
                        className="navbar-mobile-link"
                        onClick={() => setIsMenuOpen(false)}
                    >
                        GitHub
                    </a>
                    <div className="navbar-mobile-theme">
                        <ThemeToggle />
                    </div>
                    <Link
                        href="/docs/getting-started"
                        className="navbar-mobile-cta"
                        onClick={() => setIsMenuOpen(false)}
                    >
                        Get Started
                    </Link>
                </div>
            )}
        </nav>
    )
}
