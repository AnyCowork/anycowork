import Link from 'next/link'
import {
    ArrowRight,
    Bot,
    Plug,
    Shield,
    Zap,
    Sparkles,
    Terminal
} from 'lucide-react'

export default function LandingPage() {
    return (
        <div className="flex flex-col min-h-[calc(100vh-64px)] bg-white dark:bg-black text-gray-900 dark:text-white selection:bg-blue-500/30 font-sans transition-colors duration-300">
            {/* Background Gradients */}
            <div className="fixed inset-0 z-0 pointer-events-none overflow-hidden">
                <div className="absolute top-0 left-1/4 w-96 h-96 bg-blue-600/10 dark:bg-blue-600/20 rounded-full blur-[128px]" />
                <div className="absolute bottom-0 right-1/4 w-96 h-96 bg-purple-600/5 dark:bg-purple-600/10 rounded-full blur-[128px]" />
            </div>

            <div className="relative z-10 flex flex-col">
                {/* Hero Section */}
                <section className="relative pt-20 pb-16 px-6 sm:px-8 lg:px-12 flex flex-col items-center text-center">
                    <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full bg-blue-50 dark:bg-white/5 border border-blue-100 dark:border-white/10 text-xs font-medium text-blue-600 dark:text-blue-300 mb-8 animate-fade-in-up">
                        <Sparkles size={12} />
                        <span>Now with Gemini 3 Pro Support</span>
                    </div>

                    <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold tracking-tight mb-8 max-w-4xl mx-auto leading-tight animate-fade-in-up delay-100 text-gray-900 dark:text-white">
                        Smart. Safe. Optimized. <br />
                        <span className="text-transparent bg-clip-text bg-gradient-to-r from-blue-600 via-purple-600 to-blue-600 dark:from-blue-400 dark:via-purple-400 dark:to-white">
                            Your AI Co-worker
                        </span>
                    </h1>

                    <p className="text-lg sm:text-xl text-gray-600 dark:text-gray-400 max-w-2xl mx-auto mb-10 leading-relaxed animate-fade-in-up delay-200">
                        The intelligent workspace that understands your code, protects your privacy,
                        and accelerates your workflow with powerful agentic capabilities.
                    </p>

                    <div className="flex flex-wrap items-center justify-center gap-4 animate-fade-in-up delay-300">
                        <Link href="/docs/getting-started" className="group relative px-8 py-3.5 bg-blue-600 hover:bg-blue-500 text-white font-semibold rounded-lg transition-all flex items-center gap-2 shadow-[0_4px_20px_rgba(37,99,235,0.2)] dark:shadow-[0_0_20px_rgba(37,99,235,0.3)] hover:shadow-[0_6px_25px_rgba(37,99,235,0.3)] dark:hover:shadow-[0_0_30px_rgba(37,99,235,0.5)]">
                            Get Started
                            <ArrowRight size={18} className="group-hover:translate-x-1 transition-transform" />
                        </Link>
                        <a href="https://github.com/AnyCowork/AnyCowork" target="_blank" rel="noopener noreferrer" className="px-8 py-3.5 bg-gray-50 dark:bg-white/5 hover:bg-gray-100 dark:hover:bg-white/10 border border-gray-200 dark:border-white/10 text-gray-900 dark:text-white font-semibold rounded-lg transition-all flex items-center gap-2 backdrop-blur-sm">
                            <svg height="20" width="20" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z" />
                            </svg>
                            Star on GitHub
                        </a>
                    </div>

                    <div className="mt-20 w-full max-w-5xl mx-auto rounded-xl border border-gray-200 dark:border-white/10 bg-white dark:bg-white/5 p-2 shadow-2xl dark:backdrop-blur-sm animate-fade-in-up delay-500">
                        <img
                            src="/screenshot.png"
                            alt="AnyCowork Interface"
                            className="w-full h-auto rounded-lg shadow-inner bg-gray-100 dark:bg-gray-900"
                        />
                    </div>
                </section>

                {/* Features Bento Grid */}
                <section className="py-24 px-6 sm:px-8 lg:px-12 max-w-7xl mx-auto w-full">
                    <div className="text-center mb-16">
                        <h2 className="text-3xl sm:text-4xl font-bold mb-4 text-gray-900 dark:text-white">Why AnyCowork?</h2>
                        <p className="text-gray-600 dark:text-gray-400">Built for power users who value intelligence, safety, and speed.</p>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                        {/* Large Card */}
                        <div className="md:col-span-2 p-8 rounded-2xl bg-gray-50 dark:bg-white/5 border border-gray-100 dark:border-white/10 hover:border-blue-500/30 transition-colors group">
                            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-500/20 rounded-xl flex items-center justify-center text-blue-600 dark:text-blue-400 mb-6 group-hover:scale-110 transition-transform duration-300">
                                <Bot size={24} />
                            </div>
                            <h3 className="text-xl font-bold mb-3 text-gray-900 dark:text-white">Smart & Flexible</h3>
                            <p className="text-gray-600 dark:text-gray-400 leading-relaxed">
                                Don't get locked into one intelligence. Switch seamlessly between Gemini 3 Pro, GPT-5, Claude 3.5 Sonnet,
                                or local models like Llama 3 for the perfect balance of smarts and speed.
                            </p>
                        </div>

                        {/* Normal Card */}
                        <div className="p-8 rounded-2xl bg-gray-50 dark:bg-white/5 border border-gray-100 dark:border-white/10 hover:border-purple-500/30 transition-colors group">
                            <div className="w-12 h-12 bg-purple-100 dark:bg-purple-500/20 rounded-xl flex items-center justify-center text-purple-600 dark:text-purple-400 mb-6 group-hover:scale-110 transition-transform duration-300">
                                <Plug size={24} />
                            </div>
                            <h3 className="text-xl font-bold mb-3 text-gray-900 dark:text-white">Highly Optimized</h3>
                            <p className="text-gray-600 dark:text-gray-400 leading-relaxed">
                                MCP Native architecture connects your agent directly to databases and tools, eliminating copy-paste workflows and context switching.
                            </p>
                        </div>

                        {/* Normal Card */}
                        <div className="p-8 rounded-2xl bg-gray-50 dark:bg-white/5 border border-gray-100 dark:border-white/10 hover:border-green-500/30 transition-colors group">
                            <div className="w-12 h-12 bg-green-100 dark:bg-green-500/20 rounded-xl flex items-center justify-center text-green-600 dark:text-green-400 mb-6 group-hover:scale-110 transition-transform duration-300">
                                <Shield size={24} />
                            </div>
                            <h3 className="text-xl font-bold mb-3 text-gray-900 dark:text-white">Safe by Design</h3>
                            <p className="text-gray-600 dark:text-gray-400 leading-relaxed">
                                Your keys and code never leave your machine unless you want them to. Control exactly what data is shared with online providers.
                            </p>
                        </div>

                        {/* Large Card */}
                        <div className="md:col-span-2 p-8 rounded-2xl bg-gray-50 dark:bg-white/5 border border-gray-100 dark:border-white/10 hover:border-pink-500/30 transition-colors group">
                            <div className="w-12 h-12 bg-pink-100 dark:bg-pink-500/20 rounded-xl flex items-center justify-center text-pink-600 dark:text-pink-400 mb-6 group-hover:scale-110 transition-transform duration-300">
                                <Zap size={24} />
                            </div>
                            <h3 className="text-xl font-bold mb-3 text-gray-900 dark:text-white">Agentic Workflows</h3>
                            <p className="text-gray-600 dark:text-gray-400 leading-relaxed">
                                Go beyond simple chat. AnyCowork's Planner-Worker architecture breaks down complex objectives (like "refactor this module")
                                into executed steps, editing files and running commands for you.
                            </p>
                        </div>
                    </div>
                </section>

                {/* How it works */}
                <section className="py-24 px-6 sm:px-8 lg:px-12 border-t border-gray-100 dark:border-white/5 bg-gray-50/[0.3] dark:bg-white/[0.02]">
                    <div className="max-w-7xl mx-auto">
                        <h2 className="text-3xl sm:text-4xl font-bold mb-16 text-center text-gray-900 dark:text-white">How It Works</h2>

                        <div className="grid grid-cols-1 md:grid-cols-3 gap-12 relative">
                            {/* Connecting Line (Desktop) */}
                            <div className="hidden md:block absolute top-12 left-[16%] right-[16%] h-0.5 bg-gradient-to-r from-blue-500/20 via-purple-500/20 to-blue-500/20 z-0" />

                            <div className="relative z-10 flex flex-col items-center text-center">
                                <div className="w-24 h-24 rounded-full bg-white dark:bg-gray-900 border border-gray-200 dark:border-white/10 flex items-center justify-center mb-6 shadow-xl">
                                    <span className="text-2xl font-bold text-blue-600 dark:text-blue-400">1</span>
                                </div>
                                <h3 className="text-xl font-bold mb-2 text-gray-900 dark:text-white">Connect</h3>
                                <p className="text-gray-600 dark:text-gray-400">Add your API keys and connect your local tools via MCP.</p>
                            </div>

                            <div className="relative z-10 flex flex-col items-center text-center">
                                <div className="w-24 h-24 rounded-full bg-white dark:bg-gray-900 border border-gray-200 dark:border-white/10 flex items-center justify-center mb-6 shadow-xl">
                                    <span className="text-2xl font-bold text-purple-600 dark:text-purple-400">2</span>
                                </div>
                                <h3 className="text-xl font-bold mb-2 text-gray-900 dark:text-white">Collaborate</h3>
                                <p className="text-gray-600 dark:text-gray-400">Chat with the agent, ask questions, or assign complex coding tasks.</p>
                            </div>

                            <div className="relative z-10 flex flex-col items-center text-center">
                                <div className="w-24 h-24 rounded-full bg-white dark:bg-gray-900 border border-gray-200 dark:border-white/10 flex items-center justify-center mb-6 shadow-xl">
                                    <span className="text-2xl font-bold text-green-600 dark:text-green-400">3</span>
                                </div>
                                <h3 className="text-xl font-bold mb-2 text-gray-900 dark:text-white">Execute</h3>
                                <p className="text-gray-600 dark:text-gray-400">Approve proposed changes and watch the agent write code and run tests.</p>
                            </div>
                        </div>
                    </div>
                </section>

                {/* Footer */}
                <footer className="py-12 px-6 border-t border-gray-200 dark:border-white/10 bg-gray-50 dark:bg-black">
                    <div className="max-w-7xl mx-auto flex flex-col md:flex-row justify-between items-center gap-6">
                        <div className="text-sm text-gray-500 dark:text-gray-500">
                            © {new Date().getFullYear()} AnyCowork. Open source under MIT License.
                        </div>
                        <div className="flex gap-6">
                            <a href="https://github.com/AnyCowork/AnyCowork" className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors">GitHub</a>
                            <a href="/docs" className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors">Documentation</a>
                            <a href="/blog" className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white transition-colors">Blog</a>
                        </div>
                    </div>
                </footer>
            </div>

            <style jsx global>{`
        @keyframes fade-in-up {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }
        .animate-fade-in-up {
          opacity: 0;
          animation: fade-in-up 0.8s ease-out forwards;
        }
        .delay-100 { animation-delay: 0.1s; }
        .delay-200 { animation-delay: 0.2s; }
        .delay-300 { animation-delay: 0.3s; }
        .delay-500 { animation-delay: 0.5s; }
      `}</style>
        </div>
    )
}
