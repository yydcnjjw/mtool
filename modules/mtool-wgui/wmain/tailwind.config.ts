import type { Config } from 'tailwindcss'
import * as md3 from './tailwind-plugins/md3'

import tailwindcssFomrs from '@tailwindcss/forms'
import containerQueries  from './tailwind-plugins/container-queries'

export default {
    content: [
        '../../../crates/**/*.rs',
        '../../../modules/**/*.rs'
    ],
    theme: {
        extend: {
            colors: md3.colors('dark'),
            fontFamily: {
                'mono': ['Hack', 'Consolas', 'ui-monospace']
            }
        },
    },
    plugins: [
        tailwindcssFomrs,
        containerQueries
    ],
} satisfies Config

