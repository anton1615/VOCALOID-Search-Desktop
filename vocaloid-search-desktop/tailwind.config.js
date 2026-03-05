/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./pip.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        primary: '#14b8a6',
        surface: '#1a1a2e',
        background: '#0f0f1a',
      }
    },
  },
  plugins: [],
}
