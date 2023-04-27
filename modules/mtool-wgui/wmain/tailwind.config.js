/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "../core/src/**/*.rs",
    "../../mtool-interactive/wgui/src/**/*.rs"
  ],
  theme: {
    extend: {},
    fontFamily: {
      'mono': ['Hack', 'Consolas', 'ui-monospace']
    }
  },
  plugins: [
    "@tailwindcss/forms"
  ],
}
