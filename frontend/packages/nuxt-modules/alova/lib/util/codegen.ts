/// <reference types="node" />

/**
 * Code generation utility functions (shared by the defaults/to plugins)
 */

import fs from 'node:fs'
import path from 'node:path'

// ─── File writing ──────────────────────────────────────────────────────────────

/**
 * Write generated code to the target file, creating parent directories automatically
 */
export const writeGeneratedFile = (outputDir: string, filename: string, code: string): void => {
  const outPath = path.join(outputDir, filename)
  fs.mkdirSync(path.dirname(outPath), { recursive: true })
  fs.writeFileSync(outPath, code)
}

// ─── Indentation ──────────────────────────────────────────────────────────────

export const indent = (n: number) => ' '.repeat(n)

// ─── If chain generation ──────────────────────────────────────────────────────

/**
 * Generate an if-chain code block for a discriminated union
 *
 * Iterate variants in reverse: the last one (originally the first) gets no if condition and returns directly as fallback.
 *
 * @param items variant list (order matches the original variants)
 * @param buildCond generates an if condition expression, e.g. `obj?.type === "coil"`
 * @param buildReturn generates the return expression body
 * @param indentSize number of indent spaces, default 4
 */
export const buildIfChain = <T>(
  items: T[],
  buildCond: (item: T) => string,
  buildReturn: (item: T) => string,
  indentSize = 4,
): string => {
  const pad = indent(indentSize)
  return [...items]
    .reverse()
    .map((item, i) => {
      const ret = `return ${buildReturn(item)}`
      if (i === items.length - 1) return `${pad}${ret}`
      return `${pad}if (${buildCond(item)}) { ${ret} }`
    })
    .join('\n')
}
