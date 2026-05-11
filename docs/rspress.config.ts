import * as path from 'node:path';
import { defineConfig } from '@rspress/core';

export default defineConfig({
  root: path.join(__dirname, 'docs'),
  title: 'Safi UI',
  description:
    'Declarative XML-driven mobile UI framework in pure Rust, native Vulkan and Metal via SDL_GPU.',
  // icon: '/rspress-icon.png',
  // logo: {
  //   light: '/rspress-light-logo.png',
  //   dark: '/rspress-dark-logo.png',
  // },
  themeConfig: {
    socialLinks: [
      {
        icon: 'github',
        mode: 'link',
        content: 'https://github.com/AbdulKaderSafi/safi-ui',
      },
    ],
    footer: {
      message: 'Safi-UI - Safi Studio - MIT License',
    },
  },
});
