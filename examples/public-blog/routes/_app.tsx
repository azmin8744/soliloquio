import { type PageProps } from "$fresh/server.ts";
import Footer from "../components/Footer.tsx";

export default function App({ Component }: PageProps) {
  const siteTitle = Deno.env.get("SITE_TITLE") ?? "Blog";
  return (
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link rel="stylesheet" href="/styles.css" />
      </head>
      <body class="bg-white text-gray-900">
        <Component />
        <Footer siteTitle={siteTitle} />
      </body>
    </html>
  );
}
