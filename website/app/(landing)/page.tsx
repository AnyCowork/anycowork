'use client'

import React from 'react'
import { Bot, Plug, Palette, Zap, Shield, Code, Terminal, Sparkles, ArrowRight } from 'lucide-react'

export default function LandingPage() {
  return (
    <div className="landing-page">
      {/* Hero Section */}
      <section className="hero-section">
        <div className="hero-badge">
          <span>Open Source • Local-First • Multi-Provider</span>
        </div>

        <h1 className="hero-title">
          Build Your Local-First<br />
          <span className="gradient-text">AI Coworker</span>
        </h1>

        <p className="hero-subtitle">
          An open-source, agentic platform powered by <strong>Gemini 3 Pro</strong> and <strong>MCP</strong>.
          Use any LLM, connect any tool, and keep your data on your machine.
        </p>

        <div className="hero-cta">
          <a href="/docs/getting-started" className="btn-primary btn-large">
            Get Started <ArrowRight size={20} />
          </a>
          <a href="https://github.com/AnyCowork/AnyCowork" className="btn-secondary btn-large">
            <svg height="20" width="20" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
            </svg>
            View on GitHub
          </a>
        </div>

        <div className="hero-stats">
          <div className="stat-item">
            <div className="stat-value">3+</div>
            <div className="stat-label">AI Providers</div>
          </div>
          <div className="stat-divider"></div>
          <div className="stat-item">
            <div className="stat-value">MIT</div>
            <div className="stat-label">Licensed</div>
          </div>
          <div className="stat-divider"></div>
          <div className="stat-item">
            <div className="stat-value">100%</div>
            <div className="stat-label">Open Source</div>
          </div>
        </div>
      </section>

      {/* Quick Install Section */}
      {/* Quick Install Section */}
      <section className="install-section">
        <div className="install-card">
          <Terminal size={24} className="install-icon" />
          <div className="install-content">
            <h3>Get Started</h3>
            <div className="code-snippet">
              <code>git clone https://github.com/AnyCowork/AnyCowork</code>
              <button
                className="copy-btn"
                onClick={() => navigator.clipboard.writeText('git clone https://github.com/AnyCowork/AnyCowork')}
              >
                Copy
              </button>
            </div>
            <p className="install-note">
              Then run <code>npm run tauri dev</code> to start the app.
              <br />
              <a href="/docs/getting-started">View full instructions</a>
            </p>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="features-section">
        <div className="section-header">
          <h2 className="section-title">Why AnyCowork?</h2>
          <p className="section-subtitle">Everything you need to build production-ready AI agents</p>
        </div>

        <div className="features-grid">
          <div className="feature-card">
            <div className="feature-icon">
              <Bot size={28} />
            </div>
            <h3>Multi-Provider AI</h3>
            <p>Switch seamlessly between Claude, GPT, and Gemini. Use the best model for each task without vendor lock-in.</p>
          </div>

          <div className="feature-card">
            <div className="feature-icon">
              <Sparkles size={28} />
            </div>
            <h3>Agentic Workflow</h3>
            <p>Smart Coordinator-Worker architecture. The Planner breaks down complex goals, and Workers execute them step-by-step.</p>
          </div>

          <div className="feature-card">
            <div className="feature-icon">
              <Plug size={28} />
            </div>
            <h3>MCP Native</h3>
            <p>Built on the Model Context Protocol. Connect to databases, git repos, and external tools with a standardized interface.</p>
          </div>

          <div className="feature-card">
            <div className="feature-icon">
              <Shield size={28} />
            </div>
            <h3>Safe & Private</h3>
            <p>Your data stays local. Safety by Design with granular permissions and human-in-the-loop confirmation for critical actions.</p>
          </div>

          <div className="feature-card">
            <div className="feature-icon">
              <Zap size={28} />
            </div>
            <h3>Optimized</h3>
            <p>Built with Rust & Tauri. Blazing fast performance, tiny footprint, and optimized for your desktop hardware.</p>
          </div>

          <div className="feature-card">
            <div className="feature-icon">
              <Palette size={28} />
            </div>
            <h3>Beautiful UI</h3>
            <p>Modern, clean interface inspired by the best design systems. Fully customizable and accessible.</p>
          </div>
        </div>
      </section>

      {/* Code Example Section */}
      <section className="code-section">
        <div className="section-header">
          <h2 className="section-title">Built for Developers</h2>
          <p className="section-subtitle">Simple, powerful API that gets out of your way</p>
        </div>

        <div className="code-example-container">
          <div className="code-example">
            <pre><code>{`# Create an agent with custom tools
from anycowork import Agent, Tool

@Tool
def search_docs(query: str) -> str:
    """Search documentation"""
    return search_engine.query(query)

agent = Agent(
    provider="anthropic",
    model="claude-sonnet-4",
    tools=[search_docs]
)

response = await agent.execute("Find info about federation")`}</code></pre>
          </div>

          <div className="code-features">
            <div className="code-feature-item">
              <Zap size={20} />
              <div>
                <h4>Fast Setup</h4>
                <p>Get started in minutes with intuitive APIs</p>
              </div>
            </div>
            <div className="code-feature-item">
              <Shield size={20} />
              <div>
                <h4>Type Safe</h4>
                <p>Full TypeScript and Python type support</p>
              </div>
            </div>
            <div className="code-feature-item">
              <Code size={20} />
              <div>
                <h4>Well Documented</h4>
                <p>Comprehensive guides and API references</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Testimonial Section */}
      <section className="testimonial-section">
        <div className="testimonial-card">
          <div className="quote-icon">"</div>
          <blockquote>
            AnyCowork's federation capabilities let us distribute AI workloads across our infrastructure seamlessly.
            The local-first approach gives us complete control over our data.
          </blockquote>
          <div className="testimonial-author">
            <div className="author-avatar">ET</div>
            <div>
              <div className="author-name">Engineering Team</div>
              <div className="author-role">Enterprise User</div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="cta-section">
        <div className="cta-card">
          <h2>Open Source & Free Forever</h2>
          <p>
            AnyCowork is MIT licensed. Use it for personal projects, commercial applications,
            or anything in between. No strings attached.
          </p>
          <div className="cta-buttons">
            <a href="/docs/getting-started" className="btn-primary btn-large">
              Start Building <ArrowRight size={20} />
            </a>
            <a href="/docs" className="btn-secondary btn-large">
              Read the Docs
            </a>
          </div>
        </div>
      </section>
    </div>
  )
}
