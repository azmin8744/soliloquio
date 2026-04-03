interface Props {
  title: string;
}

export default function SiteTitle({ title }: Props) {
  return (
    <div class="mb-10">
      <h1 class="text-3xl font-bold text-gray-900">{title}</h1>
    </div>
  );
}
