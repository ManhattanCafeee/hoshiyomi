export type NoPromise<T> = T extends Promise<unknown> ? never : T
