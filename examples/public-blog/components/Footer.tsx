interface Props {
  siteTitle: string;
}

export default function Footer({ siteTitle }: Props) {
  const year = new Date().getFullYear();
  return (
    <footer class="border-t border-gray-200 mt-16 py-8 text-center text-sm text-gray-500">
      <p class="font-medium text-gray-700 mb-1">{siteTitle}</p>
      <p>© {year} {siteTitle}. All rights reserved.</p>
    </footer>
  );
}
