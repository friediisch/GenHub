import { addDynamicIconSelectors } from '@iconify/tailwind'

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			colors: {
				'sidebar-gray': 'rgb(23, 23, 23)',
				gray2: 'rgb(47, 47, 47)',
				'chat-window-gray': 'rgb(33, 33, 33)',
				white: 'rgb(236, 236, 236)',
				codefontcolor: 'rgb(210,187, 133)',
			},
			keyframes: {
				flyAndFade: {
					'0%': {
						opacity: '0',
						transform: 'translateY(10px)',
					},
					'100%': {
						opacity: '1',
						transform: 'translateY(0)',
					},
				},
				ping: {
					'0%, 100%': {
						transform: 'scale(0.5)',
						opacity: 100,
					},
					'50%': {
						transform: 'scale(1.5)',
						opacity: 100,
					},
				},
			},
			animation: {
				'fly-and-fade': 'flyAndFade 0.2s ease-out forwards',
				ping: 'ping 2s cubic-bezier(0, 0, 0.2, 1) infinite',
			},
		},
	},
	plugins: [addDynamicIconSelectors()],
}
