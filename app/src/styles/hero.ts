import { heroui } from '@heroui/theme';

export default heroui({
  themes: {
    dark: {
      colors: {
        /// https://catppuccin.com/palette#flavor-mocha
        background: '#1e1e2e', /// Base
        content1: '#181825', /// Mantle
        content2: '#1e1e2e', /// Base
        content3: '#313244', /// Surface 0
        content4: '#45475a', /// Surface 1
        /// Default colors as same as foreground
        default: {
          100: '#181825', /// Mantle
          200: '#1e1e2e', /// Base
          300: '#313244', /// Surface 0
          400: '#45475a', /// Surface 1
          50: '#11111b', /// Crust
          500: '#585b70', /// Surface 2
          600: '#6c7086', /// Overlay 0
          700: '#7f849c', /// Overlay 1
          800: '#9399b2', /// Overlay 2
          900: '#a6adc8', /// Subtext 0
          DEFAULT: '#313244', /// Surface 0
          foreground: '#cdd6f4', /// Text
        },
        foreground: {
          100: '#181825', /// Mantle
          200: '#1e1e2e', /// Base
          300: '#313244', /// Surface 0
          400: '#45475a', /// Surface 1
          50: '#11111b', /// Crust
          500: '#585b70', /// Surface 2
          600: '#6c7086', /// Overlay 0
          700: '#7f849c', /// Overlay 1
          800: '#9399b2', /// Overlay 2
          900: '#a6adc8', /// Subtext 0
          DEFAULT: '#cdd6f4', /// Text
        },
      },
    },
    light: {
      colors: {
        /// https://catppuccin.com/palette#flavor-latte
        background: '#eff1f5', /// Base
        content1: '#e6e9ef', /// Mantle
        content2: '#eff1f5', /// Base
        content3: '#ccd0da', /// Surface 0
        content4: '#bcc0cc', /// Surface 1
        default: {
          100: '#e6e9ef', /// Mantle
          200: '#eff1f5', /// Base
          300: '#ccd0da', /// Surface 0
          400: '#bcc0cc', /// Surface 1
          50: '#dce0e8', /// Crust
          500: '#acb0be', /// Surface 2
          600: '#9ca0b0', /// Overlay 0
          700: '#8c8fa1', /// Overlay 1
          800: '#7c7f93', /// Overlay 2
          900: '#6c6f85', /// Subtext 0
          DEFAULT: '#ccd0da', /// Surface 0
          foreground: '#4c4f69', /// Text
        },
        foreground: {
          100: '#e6e9ef', /// Mantle
          200: '#eff1f5', /// Base
          300: '#ccd0da', /// Surface 0
          400: '#bcc0cc', /// Surface 1
          50: '#dce0e8', /// Crust
          500: '#acb0be', /// Surface 2
          600: '#9ca0b0', /// Overlay 0
          700: '#8c8fa1', /// Overlay 1
          800: '#7c7f93', /// Overlay 2
          900: '#6c6f85', /// Subtext 0
          DEFAULT: '#4c4f69', /// Text
        },
      },
    },
  },
});
