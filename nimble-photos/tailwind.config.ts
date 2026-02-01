import type { Config } from 'tailwindcss';

const config: Config = {
  content: ['./src/**/*.{html,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'Space Grotesk', 'system-ui', 'sans-serif'],
      },
      colors: {
        'ink-950': '#010409',
        'ink-900': '#020617',
        'ink-800': '#0f172a',
        'accent-gold': '#facc15',
        'accent-lime': '#a3e635',
      },
      boxShadow: {
        'gallery-card': '0 20px 45px rgba(2, 6, 23, 0.6)',
      },
    },
  },
  plugins: [],
};

export default config;
