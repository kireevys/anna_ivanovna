/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs", "./index.html"],
  theme: {
    extend: {},
  },
  plugins: [require("daisyui")],
  daisyui: {
    themes: [
      "light", "dark", "forest", "cupcake", "bumblebee", "emerald",
      "corporate", "synthwave", "retro", "cyberpunk", "valentine",
      "halloween", "garden", "aqua", "lofi", "pastel", "fantasy",
      "wireframe", "black", "luxury", "dracula", "cmyk", "autumn",
      "business", "acid", "lemonade", "night", "coffee", "winter",
    ],
  },
};
