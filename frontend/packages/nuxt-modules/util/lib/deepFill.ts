/* eslint-disable @typescript-eslint/no-explicit-any */

/**
 * Recursively fill: when a value in target is null or undefined, use the corresponding value from source
 */
export function deepFill<T>(target: T, source: T | null): T {
  // If source has no value, return target as-is
  if (source == null) return target

  // Ensure target is an object, otherwise recursion is impossible
  if (typeof target !== 'object' || target === null) {
    return target ?? source
  }

  for (const key in source) {
    if (Object.prototype.hasOwnProperty.call(source, key)) {
      const targetVal = target[key as keyof T]
      const sourceVal = source[key]

      if (targetVal === null || targetVal === undefined) {
        // Case A: target has no value; take source's value directly (including source's subtree)
        target[key as keyof T] = sourceVal as any
      } else if (
        typeof targetVal === 'object' &&
        typeof sourceVal === 'object' &&
        !Array.isArray(targetVal) // arrays are usually overwritten directly; recursive element merging is not recommended
      ) {
        // Case B: both are objects; recurse downward
        deepFill(targetVal, sourceVal as any)
      }
    }
  }
  return target
}
