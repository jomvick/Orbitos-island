/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        obsidian: {
          950: "#050505",
          900: "#0A0A0A",
          800: "#121212",
          700: "#1A1A1A",
        },
        accent: {
          blue: "#3B82F6",
          green: "#10B981",
          purple: "#8B5CF6",
          red: "#EF4444",
        },
        text: {
          primary: "#FFFFFF",
          secondary: "rgba(255, 255, 255, 0.7)",
          muted: "rgba(255, 255, 255, 0.4)",
          dim: "rgba(255, 255, 255, 0.2)",
        },
        glass: {
          border: "rgba(255, 255, 255, 0.08)",
          "border-bright": "rgba(255, 255, 255, 0.12)",
        },
      },
      fontFamily: {
        sans: ['"Inter"', "ui-sans-serif", "system-ui"],
        mono: ['"JetBrains Mono"', "ui-monospace", "monospace"],
      },
      boxShadow: {
        premium: "0 20px 50px rgba(0, 0, 0, 0.5)",
        glow: "0 0 20px rgba(59, 130, 246, 0.15)",
        "glow-accent": "0 0 15px var(--glow-color)",
      },
      animation: {
        "pulse-slow": "pulse 4s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "float": "float 3s ease-in-out infinite",
      },
      keyframes: {
        float: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-5px)" },
        },
      },
    },
  },
  plugins: [],
};
