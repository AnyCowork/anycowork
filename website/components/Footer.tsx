import { Github, Twitter, MessageCircle, Mail } from 'lucide-react'

const Link = ({ href, children }: { href: string; children: React.ReactNode }) => <a href={href}>{children}</a>;

export default function Footer() {
    const currentYear = new Date().getFullYear()

    return (
        <footer className="footer">
            <div className="footer-container">
                {/* Main Footer Content */}
                <div className="footer-grid">
                    {/* Brand Section */}
                    <div className="footer-brand">
                        <div className="footer-logo">
                            <img src="/logo.svg" alt="AnyCowork" className="footer-logo-img" />
                            <span className="footer-logo-text">AnyCowork</span>
                        </div>
                        <p className="footer-tagline">
                            Your AI coworker, powered by Gemini 3 Pro. Local-first data storage, trusted cloud AI, open source.
                        </p>
                        <div className="footer-social">
                            <a
                                href="https://github.com/AnyCowork/AnyCowork"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="footer-social-link"
                                aria-label="GitHub"
                            >
                                <Github size={20} />
                            </a>
                            <a
                                href="https://twitter.com/anycowork"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="footer-social-link"
                                aria-label="Twitter"
                            >
                                <Twitter size={20} />
                            </a>
                            <a
                                href="https://discord.gg/anycowork"
                                target="_blank"
                                rel="noopener noreferrer"
                                className="footer-social-link"
                                aria-label="Discord"
                            >
                                <MessageCircle size={20} />
                            </a>
                            <a
                                href="mailto:hello@anycowork.com"
                                className="footer-social-link"
                                aria-label="Email"
                            >
                                <Mail size={20} />
                            </a>
                        </div>
                    </div>

                    {/* Product Links */}
                    <div className="footer-section">
                        <h3 className="footer-section-title">Product</h3>
                        <ul className="footer-links">
                            <li><Link href="/docs/getting-started">Getting Started</Link></li>
                            <li><Link href="https://github.com/AnyCowork/AnyCowork/releases">Download</Link></li>
                            <li><Link href="/docs/desktop">Desktop App</Link></li>
                            <li><Link href="/docs/vision">Vision & Roadmap</Link></li>
                        </ul>
                    </div>

                    {/* Resources Links */}
                    <div className="footer-section">
                        <h3 className="footer-section-title">Resources</h3>
                        <ul className="footer-links">
                            <li><Link href="/docs">Documentation</Link></li>
                            <li><Link href="/docs/architecture">Architecture</Link></li>
                            <li><Link href="/docs/blog">Blog</Link></li>
                            <li><Link href="https://github.com/AnyCowork/AnyCowork/discussions">Community</Link></li>
                            <li><Link href="https://github.com/AnyCowork/AnyCowork/issues">Support</Link></li>
                        </ul>
                    </div>

                    {/* Open Source */}
                    <div className="footer-section">
                        <h3 className="footer-section-title">Open Source</h3>
                        <ul className="footer-links">
                            <li><Link href="/docs/blog">Blog</Link></li>
                            <li><Link href="https://github.com/AnyCowork/AnyCowork">GitHub</Link></li>
                            <li><Link href="https://github.com/AnyCowork/AnyCowork/blob/main/LICENSE">MIT License</Link></li>
                            <li><Link href="https://github.com/AnyCowork/AnyCowork/blob/main/CONTRIBUTING.md">Contributing</Link></li>
                            <li><Link href="https://gemini3.devpost.com/">Gemini 3 Hackathon</Link></li>
                        </ul>
                    </div>
                </div>

                {/* Bottom Bar */}
                <div className="footer-bottom">
                    <div className="footer-bottom-content">
                        <p className="footer-copyright">
                            © {currentYear} AnyCowork. Open source under MIT License.
                        </p>
                        <div className="footer-bottom-links">
                            <a href="https://aistudio.google.com/" target="_blank" rel="noopener noreferrer">Powered by Gemini 3 Pro</a>
                            <Link href="/docs/vision">Roadmap</Link>
                            <a href="https://github.com/AnyCowork/AnyCowork/issues" target="_blank" rel="noopener noreferrer">Report Issue</a>
                        </div>
                    </div>
                </div>
            </div>
        </footer>
    )
}
