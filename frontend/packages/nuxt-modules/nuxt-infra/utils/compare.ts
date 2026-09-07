export const compareLocalely =
  <T>(key: (x: T) => string) =>
  (a: T, b: T) =>
    key(a).localeCompare(key(b), 'zh')
