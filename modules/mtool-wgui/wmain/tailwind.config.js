/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "../../../**/*.rs"
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

