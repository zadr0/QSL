// @ts-check
import { readFileSync } from 'node:fs';
import { defineEcConfig } from '@astrojs/starlight/expressive-code';

// TextMate grammar for .qsl schemas and .deq config files.
const qslGrammar = JSON.parse(
	readFileSync(new URL('./src/grammars/qsl.tmLanguage.json', import.meta.url), 'utf8'),
);

export default defineEcConfig({
	themes: ['github-dark-dimmed', 'github-light'],
	shiki: {
		langs: [qslGrammar],
	},
	defaultProps: {
		// Shell commands get the same frame as other code. A terminal frame
		// without a title would show an empty title bar.
		overridesByLang: {
			'bash,sh,shell,zsh,powershell,ps': { frame: 'code' },
		},
	},
	styleOverrides: {
		borderRadius: '12px',
		borderColor: 'var(--sl-color-hairline-light)',
		// Code sits on the site's own surface color instead of the theme's gray.
		codeBackground: 'var(--qsl-surface)',
		codeFontSize: '0.8125rem',
		codeLineHeight: '1.7',
		codePaddingBlock: '1rem',
		codePaddingInline: '1.125rem',
		// Titles are labels, not code: set them in the text face.
		uiFontFamily: 'var(--__sl-font)',
		uiFontSize: '0.8125rem',
		uiFontWeight: '500',
		frames: {
			// A title is a small label on the code's own surface, not a tab: no
			// rule under it (theme.css places it and the copy button).
			editorBackground: 'var(--qsl-surface)',
			editorTabBarBackground: 'var(--qsl-surface)',
			editorTabBarBorderColor: 'transparent',
			editorTabBarBorderBottomColor: 'transparent',
			editorTabBorderRadius: '0',
			editorTabsMarginInlineStart: '0',
			editorTabsMarginBlockStart: '0',
			editorActiveTabBackground: 'transparent',
			editorActiveTabForeground: 'var(--sl-color-gray-3)',
			editorActiveTabBorderColor: 'transparent',
			editorActiveTabIndicatorTopColor: 'transparent',
			editorActiveTabIndicatorBottomColor: 'transparent',
			terminalBackground: 'var(--qsl-surface)',
			terminalTitlebarBackground: 'var(--qsl-surface)',
			terminalTitlebarForeground: 'var(--sl-color-gray-3)',
			terminalTitlebarBorderBottomColor: 'var(--sl-color-hairline-light)',
			terminalTitlebarDotsOpacity: '0',
			frameBoxShadowCssValue: 'none',
		},
		textMarkers: {
			// Marked lines are a light tint with a thin edge, in the accent's hue,
			// so the code inside stays the thing you read.
			insHue: '165',
			backgroundOpacity: '22%',
			lineMarkerAccentWidth: '2px',
		},
	},
});
