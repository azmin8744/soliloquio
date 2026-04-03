import { type Config } from "tailwindcss";
import typography from "@tailwindcss/typography";

export default {
  content: [
    "{routes,components}/**/*.{ts,tsx,js,jsx}",
  ],
  plugins: [typography],
} satisfies Config;
