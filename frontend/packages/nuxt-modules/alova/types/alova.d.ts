export interface AlovaCustomTypeMeta {
  multipart?: boolean
}

declare module 'alova' {
  export interface AlovaCustomTypes {
    meta: AlovaCustomTypeMeta
  }
}
