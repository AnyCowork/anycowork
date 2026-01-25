import type { DocsThemeConfig } from "nextra-theme-docs";
import { useConfig } from "nextra-theme-docs";
import { useRouter } from "next/router";

const logo = (
  <>
    <div className="flex flex-row items-center justify-center">
      <img className="h-10 w-auto rounded-md" src="/logo.svg" alt="AnyCowork Logo" />
      <h1 className="text-2xl ml-2 font-bold">AnyCowork</h1>
    </div>
    <style jsx>{`
      span {
        padding: 0.5rem 0.5rem 0.5rem 0;
        mask-image: linear-gradient(
          60deg,
          black 25%,
          rgba(0, 0, 0, 0.2) 50%,
          black 75%
        );
        mask-size: 400%;
        mask-position: 0%;
      }
      span:hover {
        mask-position: 100%;
        transition: mask-position 1s ease, -webkit-mask-position 1s ease;
      }
    `}</style>
  </>
);

const config: DocsThemeConfig = {
  project: {
    link: "https://github.com/AnyCowork/AnyCowork",
  },
  docsRepositoryBase: "https://github.com/AnyCowork/AnyCowork",
  useNextSeoProps() {
    const { asPath } = useRouter();
    if (asPath !== "/") {
      return {
        titleTemplate: "%s – AnyCowork",
      };
    }
  },
  logo,
  head: function useHead() {
    const { title } = useConfig();
    const { route } = useRouter();
    const socialCard =
      route === "/" || !title
        ? "https://www.anycowork.com/og.jpeg"
        : `https://www.anycowork.com/api/og?title=${title}`;

    return (
      <>
        <meta name="msapplication-TileColor" content="#fff" />
        <meta name="theme-color" content="#fff" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta httpEquiv="Content-Language" content="en" />
        <meta
          name="description"
          content="Open-source, local-first AI assistant platform built with Rust and Tauri. Multi-provider AI, agentic workflows, MCP native."
        />
        <meta
          name="og:description"
          content="Open-source, local-first AI assistant platform built with Rust and Tauri. Multi-provider AI, agentic workflows, MCP native."
        />
        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:image" content={socialCard} />
        <meta name="twitter:site:domain" content="anycowork.com" />
        <meta name="twitter:url" content="https://www.anycowork.com" />
        <meta
          name="og:title"
          content={title ? title + " – AnyCowork" : "AnyCowork"}
        />
        <meta name="og:image" content={socialCard} />
        <meta name="apple-mobile-web-app-title" content="AnyCowork" />
        <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
        <link
          rel="icon"
          href="/favicon.svg"
          type="image/svg+xml"
          media="(prefers-color-scheme: light)"
        />
      </>
    );
  },
  banner: {
    key: "anycowork-alpha",
    text: (
      <a href="https://github.com/AnyCowork/AnyCowork" target="_blank" rel="noreferrer">
        🚀 AnyCowork is in active development - Star us on GitHub! 🚀
      </a>
    ),
  },
  editLink: {
    text: "Edit this page on GitHub →",
  },
  feedback: {
    content: "Question? Give us feedback →",
    labels: "feedback",
  },
  sidebar: {
    titleComponent({ title, type }) {
      if (type === "separator") {
        return <span className="cursor-default">{title}</span>;
      }
      return <>{title}</>;
    },
    defaultMenuCollapseLevel: 2,
    toggleButton: true,
  },
  footer: {
    text: (
      <div className="flex w-full flex-col items-center sm:items-start">
        <div>
          <a
            className="flex items-center gap-1 text-current"
            target="_blank"
            rel="noopener noreferrer"
            title="AnyCowork on GitHub"
            href="https://github.com/AnyCowork/AnyCowork"
          >
            <div className="pt-0 mt-0">
              Open Source •{" "}
              <span className="font-extrabold text-transparent bg-clip-text bg-gradient-to-r from-blue-600 to-purple-600 text-bold">
                MIT Licensed
              </span>
            </div>
          </a>
        </div>
        <p className="mt-2 text-xs">
          © {new Date().getFullYear()} The AnyCowork Project.
        </p>
      </div>
    ),
  },
};

export default config;
