/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        background: {
          DEFAULT: '#090a0f',
          light: '#f5f5f7',
        },
        surface: {
          DEFAULT: 'rgba(18, 20, 29, 0.75)',
          light: 'rgba(255, 255, 255, 0.75)',
          elevated: 'rgba(28, 31, 46, 0.85)',
        },
        border: {
          DEFAULT: 'rgba(255, 255, 255, 0.08)',
          light: 'rgba(0, 0, 0, 0.08)',
          highlight: 'rgba(255, 255, 255, 0.18)',
        },
        accent: {
          DEFAULT: '#3b82f6',
          hover: '#2563eb',
          light: '#60a5fa',
        },
      },
      backdropBlur: {
        glass: '24px',
        panel: '32px',
      },
      borderRadius: {
        '2xl': '16px',
        '3xl': '24px',
        '4xl': '28px',
      },
      boxShadow: {
        'glass': '0 8px 32px 0 rgba(0, 0, 0, 0.37)',
        'glass-sm': '0 4px 16px 0 rgba(0, 0, 0, 0.25)',
        'glass-inset': 'inset 0 1px 0 0 rgba(255, 255, 255, 0.12)',
        'glow-accent': '0 0 20px -3px rgba(59, 130, 246, 0.5)',
      },
    },
  },
  plugins: [],
}
