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
                        transform: 'translateY(-10px) scaleY(0)',
                        'transform-origin': 'top',
                    },
                    '100%': {
                        opacity: '1',
                        transform: 'translateY(0px) scaleY(1)',
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
                'shake': {
                    '0%, 100%': { transform: 'translateX(0)' },
                    '20%, 60%': { transform: 'translateX(-3px)' },
                    '40%, 80%': { transform: 'translateX(3px)' },
                },
            },
            animation: {
                'slide-down': 'slide-down 0.2s ease-out both',
                'fade-in': 'fade-in 0.4s ease-out both',
                'shake-fast': 'shake 0.5s ease-in-out both',
                'bounce-mid': 'bounce 0.8s ease-in-out both',
            },
            transitionProperty: {
                'all': 'all',
            }
        },
    },
    plugins: [],
}