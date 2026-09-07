import { defineUniPages } from '@uni-helper/vite-plugin-uni-pages'

export default defineUniPages({
  globalStyle: {
    backgroundColor: '@bgColor',
    backgroundColorBottom: '@bgColorBottom',
    backgroundColorTop: '#ffffff',
    backgroundTextStyle: '@bgTxtStyle',
    navigationBarBackgroundColor: '#ffffff',
    navigationBarTextStyle: '@navTxtStyle',
    navigationBarTitleText: 'hoshiyomi',
    navigationStyle: 'custom',
  },
  tabBar: {
    backgroundColor: '@tabBgColor',
    borderStyle: '@tabBorderStyle',
    color: '@tabFontColor',
    selectedColor: '@tabSelectedColor',
    list: [
      {
        pagePath: 'pages/home/index',
        text: '工作台',
      },
      {
        pagePath: 'pages/profile/index',
        text: '我的',
      },
    ],
  },
})
