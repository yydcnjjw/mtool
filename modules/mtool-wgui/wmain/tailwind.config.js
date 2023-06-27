/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "../../../crates/**/*.rs",
    "../../../modules/**/*.rs"
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

