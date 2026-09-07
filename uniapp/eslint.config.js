import uni from '@uni-helper/eslint-config'

export default uni({
  unocss: true,
})
  .prepend({
    ignores: ['src/api/generated/**'],
  })
  .append({
    rules: {
      'unused-imports/no-unused-vars': 'off',
      'vue/valid-template-root': 'off',
    },
  })
