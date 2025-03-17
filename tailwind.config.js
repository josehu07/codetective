/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        files: ["*.html", "./src/**/*.rs"],
        transform: {
            rs: (content) => content.replace(/(?:^|\s)class:/g, ' '),
        },
    },
    theme: {
        extend: {
            keyframes: {
                'slide-down': {
                    '0%': {
                        opacity: '0',
                        transform: 'translateY(-10px)',
                        maxHeight: '0'
                    },
                    '100%': {
                        opacity: '1',
                        transform: 'translateY(0)',
                        maxHeight: '200px'
                    },
                },
                'fade-in': {
                    '0%': {
                        opacity: '0'
                    },
                    '100%': {
                        opacity: '1'
                    }
                },
                'fade-out': {
                    '0%': {
                        opacity: '1'
                    },
                    '100%': {
                        opacity: '0'
                    }
                },
            },
            animation: {
                'slide-down': 'slide-down 0.3s ease-out both',
                'fade-in': 'fade-in 0.4s ease-out both',
                'fade-out': 'fade-out 0.4s ease-out both',
            },
            transitionProperty: {
                'all': 'all',
            }
        },
    },
    plugins: [],
}