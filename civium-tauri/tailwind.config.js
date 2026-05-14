/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        civium: {
          50:  "#f0f4ff",
          100: "#e0e9ff",
          500: "#4f6ef7",
          600: "#3a56e8",
          700: "#2d44c9",
          900: "#1a2a7a",
        },
      },
    },
  },
  plugins: [],
};
