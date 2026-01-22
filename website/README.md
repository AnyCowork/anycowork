# AnyCowork Website

Official website and documentation for AnyCowork, hosted at www.anycowork.com

## Development

Install dependencies:
```bash
npm install
```

Run development server:
```bash
npm run dev
```

Visit http://localhost:3000

## Building

Build for production:
```bash
npm run build
```

Export static site:
```bash
npm run export
```

## Structure

```
website/
├── pages/
│   ├── index.mdx           # Landing page
│   ├── features.mdx        # Features page
│   ├── pricing.mdx         # Pricing page
│   ├── blog/               # Blog posts
│   │   ├── index.mdx
│   │   └── *.mdx
│   └── docs/               # Documentation
│       ├── index.mdx
│       └── *.mdx
├── theme.config.tsx        # Nextra theme config
├── next.config.mjs         # Next.js config
└── package.json
```

## Deployment

Deploy to Vercel:
```bash
vercel
```

Or any static hosting service (Netlify, GitHub Pages, etc.)

## License

MIT
