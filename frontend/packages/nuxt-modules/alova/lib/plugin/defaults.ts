/// <reference types="node" />

import { createPlugin } from '@alova/wormhole/plugin'
import type { OpenAPIV3 } from 'openapi-types'
import {
  defaultFnName,
  fullFnName,
  getRefName,
  getTypeName,
  initFnName,
  isRef,
  isSchemaObject,
  resolveSchemaVariants,
  stripField,
  tryResolveDiscriminatedUnion,
  type DiscriminatedUnionResult,
  type DiscriminatedVariant,
  type OpenAPISchema,
  type OpenAPISchemaObject,
  type OpenAPISchemas,
} from '../util/openapi'
import { buildIfChain, writeGeneratedFile } from '../util/codegen'

// ─── Plugin options ───────────────────────────────────────────────────────────

export interface DefaultsPluginOpts {
  /** Filter schemas that do not need to be generated */
  filter?: (name: string, schema: OpenAPISchema) => boolean
  /** Custom default value expression for a field */
  customDefaultExpr?: (key: string, schema: OpenAPISchemaObject) => string | undefined
  /** Field names marked as file type (generates new Blob([])) */
  fileFieldNames?: string[]
  /** Union type discriminator field for associated objects
   * @default 'type'
   */
  unionType?: string
}

// ─── Runtime helper code (injected at the top of generated files) ────────────────

/**
 * Runtime helper code injected into the generated defaults file
 *
 * Generates three kinds of functions:
 * - `$fullXxx(opts?)` — includes all fields (required + optional), corresponds to `DeepRequired<T>`
 *   - `fullXxx()` → DeepRequired<T> (nullable fields become null)
 *   - `fullXxx({ notNull: true })` → DeepRequired<T> (nullable fields get real values, no null)
 * - `$initXxx()` — generates field default values normally per schema, returns T:
 *   - optional fields (undefined allowed) → not generated
 *   - nullable fields → null
 *   - required non-nullable fields → default value by type
 *   Mainly used to fill missing fields in toXxx and as form initial values
 * - `$defaultXxx()` — pure dict types (only additionalProperties, no named properties) return `{}`
 *   Stateless fields need initialization; return a fixed value directly
 */
const runtimeHelperCode = (extraImports: string) => `/* eslint-disable @typescript-eslint/no-explicit-any */
/* eslint-disable @typescript-eslint/unified-signatures */
import type Types from './globals'
import { defu } from 'defu'
${extraImports}
export type DeepRequired<T> = T extends object
  ? { [K in keyof T]-?: T[K] extends (infer U)[] ? DeepRequired<U>[] : T[K] extends object ? DeepRequired<T[K]> : T[K] }
  : T

export type DeepNotNull<T> = T extends object
  ? { [K in keyof T]-?: T[K] extends (infer U)[] ? DeepNotNull<U>[] : T[K] extends object ? DeepNotNull<T[K]> : NonNullable<T[K]> }
  : NonNullable<T>

type DefineFullFn<T> = {
  (): DeepRequired<T>
  (input: Partial<DeepRequired<T>>): DeepRequired<T>
  (opts: { input?: Partial<DeepNotNull<T>>; notNull: true }): DeepNotNull<T>
  (opts: { input?: Partial<DeepRequired<T>>; notNull?: false }): DeepRequired<T>
}

type DefineInitFn<T> = {
  <O extends Partial<T> & Record<string, any>>(obj?: O): T
}

const defineFull = <T>(
  fields: (notNull: any, input?: any) => DeepRequired<T>,
): DefineFullFn<T> => {
  return (arg?: any): any => {
    const { notNull, input } = arg?.notNull === undefined ? { notNull: false, input: arg } : arg as { notNull: boolean; input?: any }
    if (!input) return fields(notNull)
    return defu(input, fields(notNull, input) as any)
  }
}

const defineInit = <T>(
  fields: (obj?: any) => T,
): DefineInitFn<T> => {
  return <O extends Partial<T> & Record<string, any>>(obj?: O): T => {
    if (!obj) return fields()
    return defu(obj, fields(obj) as any) as T
  }
}
`

// ─── Layer 1: Basic value generation (primitive types → literal expressions) ─────

/**
 * Determine whether a schema can be null (nullable field)
 */
const isNullable = (schema: OpenAPISchemaObject): boolean =>
  !!(
    (schema as OpenAPIV3.SchemaObject).nullable ||
    (Array.isArray(schema.type) ? schema.type.includes('null') : schema.type === 'null') ||
    schema.oneOf?.some((v) => isSchemaObject(v) && v.type === 'null') ||
    schema.anyOf?.some((v) => isSchemaObject(v) && v.type === 'null')
  )

/**
 * Generate the real default value expression for non-null cases (ignore nullable checks, type only)
 */
const getNonNullExpr = (fieldName: string, schema: OpenAPISchemaObject, ctx: FieldGenContext): string => {
  const { opts } = ctx
  if (opts?.customDefaultExpr) {
    const custom = opts.customDefaultExpr(fieldName, schema)
    if (custom !== undefined) return custom
  }

  if (schema.default !== undefined) return JSON.stringify(schema.default)
  if (opts?.fileFieldNames?.includes(fieldName) || schema.format === 'binary') return 'new Blob([])'
  if (schema.enum?.length) return JSON.stringify(schema.enum[0])

  // Extract the actual non-null type (handles union types like string | null)
  const effectiveType = Array.isArray(schema.type)
    ? (schema.type.find((t) => t !== 'null') ?? schema.type[0])
    : schema.type

  switch (effectiveType) {
    case 'string':
      switch (schema.format) {
        case 'date-time':
          return 'new Date().toISOString()'
        case 'date':
          ctx.requiredUtils.add('dateString')
          return 'dateString()'
        case 'time':
          ctx.requiredUtils.add('timeString')
          return 'timeString()'
        case 'uuid':
          ctx.requiredImports.add('uuidv4')
          return 'uuidv4()'
        default:
          return "''"
      }
    case 'number':
    case 'integer':
      return schema.format === 'unix-time' ? 'Date.now()' : '0'
    case 'boolean':
      return 'false'
    case 'array':
      return '[]'
    case 'object':
      return '{}'
    default:
      return 'undefined'
  }
}

/**
 * Generate the default value expression for a field
 *
 * - nullable fields: generate `notNull ? <nonNullValue> : null` (decided at runtime by notNull)
 * - other fields: return a fixed value directly
 */
const getPrimitiveDefaultExpr = (fieldName: string, schema: OpenAPISchemaObject, ctx: FieldGenContext): string => {
  if (isNullable(schema)) {
    return `notNull ? ${getNonNullExpr(fieldName, schema, ctx)} : null`
  }
  return getNonNullExpr(fieldName, schema, ctx)
}

// ─── Layer 2: Field value expression generation (Schema → code expression) ───────

interface FieldGenContext {
  opts?: DefaultsPluginOpts
  /** Schema names registered as simple data types (enum / integer alias); no notNull parameter needed */
  dataTypeNames: string[]
  requiredUtils: Set<'dateString' | 'timeString'>
  requiredImports: Set<'uuidv4'>
  /** Current field access path, passed as the obj argument when calling ref for a discriminated union */
  fieldPath?: string[]
  /** Full schema definitions, used for $ref resolution (expands referenced types in flattenAllOf) */
  schemas?: OpenAPISchemas
}

type FieldMode = 'full' | 'partial'

const inPath = (segments: string[]) => `input?.${segments.join('.')}`

/**
 * Generate the default value expression for a single field
 * - $ref → call the corresponding fullXxx/initXxx function
 * - allOf/anyOf/oneOf → recurse into the first non-null variant
 * - inline object (has properties) → expand to a literal
 * - other → primitive value (nullable fields generate a conditional expression)
 */
const buildFieldExpr = (
  fieldName: string,
  schema: OpenAPISchema,
  ctx: FieldGenContext,
  indent = 4,
  mode: FieldMode = 'full',
): string => {
  if (isRef(schema)) {
    const refName = getRefName(schema.$ref)
    const isDataType = ctx.dataTypeNames.includes(refName)

    // Simple types (enum/integer alias) use defaultFn
    if (isDataType) return `${defaultFnName(refName)}()`

    if (mode === 'partial') {
      if (ctx.fieldPath) {
        return `${initFnName(refName)}(${inPath(ctx.fieldPath)})`
      }
      return `${initFnName(refName)}()`
    }

    if (ctx.fieldPath) {
      return `${fullFnName(refName)}({ notNull, input: ${inPath(ctx.fieldPath)} })`
    }
    return `${fullFnName(refName)}({ notNull })`
  }

  if (!isSchemaObject(schema)) return 'undefined'

  // allOf/anyOf/oneOf: recurse into the first non-null type variant
  const nonNullOf = (schema.allOf ?? schema.anyOf ?? schema.oneOf)?.find(
    (v) => !(isSchemaObject(v) && v.type === 'null'),
  )
  if (nonNullOf) return buildFieldExpr(fieldName, nonNullOf, ctx, indent, mode)

  if (schema.enum?.length) return JSON.stringify(schema.enum[0])

  // nullable → null (partial mode)
  if (mode === 'partial' && isNullable(schema)) return 'null'

  // Inline object (with named properties): expand to a literal
  if (schema.type === 'object' && schema.properties && Object.keys(schema.properties).length > 0) {
    return buildInlineObjectExpr(schema, ctx, indent, mode)
  }

  if (schema.type === 'array') return '[]'

  // Partial mode uses a different expression
  if (mode === 'partial') return getNonNullExpr(fieldName, schema, ctx)
  return getPrimitiveDefaultExpr(fieldName, schema, ctx)
}

/**
 * Generate a literal expression for an inline object
 */
const buildInlineObjectExpr = (
  schema: OpenAPISchemaObject,
  ctx: FieldGenContext,
  indentSize: number,
  mode: FieldMode = 'full',
): string => {
  const properties = schema.properties ?? {}
  if (Object.keys(properties).length === 0) return '{}'

  const pad = ' '.repeat(indentSize - 2)
  const innerPad = ' '.repeat(indentSize)

  // Partial mode only handles required fields
  const required = mode === 'partial' ? new Set<string>(schema.required ?? []) : null

  const lines: string[] = []
  for (const [key, prop] of Object.entries(properties)) {
    if (required && !required.has(key)) continue
    const ctxWithPath = ctx.fieldPath ? { ...ctx, fieldPath: [...ctx.fieldPath, key] } : { ...ctx, fieldPath: [key] }
    lines.push(`${innerPad}${key}: ${buildFieldExpr(key, prop, ctxWithPath, indentSize, mode)}`)
  }

  if (lines.length === 0) return '{}'
  return `{\n${lines.join(',\n')}\n${pad}}`
}

// ─── Layer 3: Generate function code construction ───────────────────────────────

/**
 * Wrap a defineFull + defineInit function pair (object body mode; body is the property lines inside `({ ... })`).
 */
const wrapFullInitExpr = (name: string, fullBody: string, initBody: string): string =>
  `
export const ${fullFnName(name)}: DefineFullFn<Types.${getTypeName(name)}> = defineFull<Types.${getTypeName(name)}>(
  (notNull, input) => ({
${fullBody}
  })
)

export const ${initFnName(name)}: DefineInitFn<Types.${getTypeName(name)}> = defineInit<Types.${getTypeName(name)}>(
  (input) => ({
${initBody}
  })
)
`

/**
 * Build defineFull/init function code for an object schema
 *
 * - full: all fields (including optional); nullable fields become null based on the notNull parameter
 * - partial: only required fields are generated, optional fields are skipped, nullable fields become null
 *   Mainly used to fill missing fields in toXxx and as form initial values
 */
const buildObjectFn = (name: string, schema: OpenAPISchemaObject, ctx: FieldGenContext): string => {
  const required = new Set<string>(schema.required ?? [])
  const fullLines: string[] = []
  const partialLines: string[] = []

  for (const [fieldName, prop] of Object.entries(schema.properties ?? {})) {
    const ctxWithPath = { ...ctx, fieldPath: [fieldName] }
    fullLines.push(`    ${fieldName}: ${buildFieldExpr(fieldName, prop, ctxWithPath, 4, 'full')}`)
    if (required.has(fieldName)) {
      partialLines.push(`    ${fieldName}: ${buildFieldExpr(fieldName, prop, ctxWithPath, 4, 'partial')}`)
    }
  }

  return wrapFullInitExpr(name, fullLines.join(',\n'), partialLines.join(',\n'))
}

// ─── Union type code construction (three mutually exclusive paths) ───────────────

/**
 * Path A: allOf [$ref, ...] forwards directly to the already-generated ref function.
 * When allOf has additional members, merge the fields then forward.
 */
const buildRefForwardFn = (name: string, schema: OpenAPISchemaObject, ctx: FieldGenContext): string => {
  const rawVariants = schema.oneOf ?? schema.anyOf ?? schema.allOf ?? []
  const firstRaw = rawVariants[0]
  if (!isRef(firstRaw)) return ''
  const refName = getRefName(firstRaw.$ref)
  const remaining = rawVariants.slice(1)

  let fullBody: string
  let partialBody: string

  if (schema.allOf && remaining.length > 0) {
    const fullLines: string[] = []
    const partialLines: string[] = []

    for (const item of remaining) {
      if (isRef(item)) {
        const rn = getRefName(item.$ref)
        fullLines.push(`    ...${fullFnName(rn)}({ notNull })`)
        partialLines.push(`    ...${initFnName(rn)}(input)`)
        continue
      }
      if (!isSchemaObject(item) || !item.properties) continue

      const required = new Set(item.required ?? [])
      for (const [key, prop] of Object.entries(item.properties)) {
        const ctxPath = { ...ctx, fieldPath: [key] }
        fullLines.push(`    ${key}: ${buildFieldExpr(key, prop, ctxPath, 4, 'full')}`)
        if (required.has(key)) {
          partialLines.push(`    ${key}: ${buildFieldExpr(key, prop, ctxPath, 4, 'partial')}`)
        }
      }
    }

    fullBody = [`    ...${fullFnName(refName)}({ notNull, ...input })`, ...fullLines].join(',\n')
    partialBody = [`    ...${initFnName(refName)}(input)`, ...partialLines].join(',\n')
  } else {
    fullBody = `    ...${fullFnName(refName)}({ notNull, ...input })`
    partialBody = `    ...${initFnName(refName)}(input)`
  }

  return wrapFullInitExpr(name, fullBody, partialBody)
}

/**
 * Path B: Discriminated union → generate an if-else chain dispatch.
 */
const buildDiscriminatedUnionFn = (name: string, du: DiscriminatedUnionResult, ctx: FieldGenContext): string => {
  const { fieldName, variants: discriminatedVariants } = du

  // Generate the inner expansion expression for a single variant
  const buildVariantInnerExpr = ({ schema: variant, refName }: DiscriminatedVariant, mode: FieldMode): string => {
    if (refName) {
      return mode === 'partial' ? `${initFnName(refName)}(input)` : `${fullFnName(refName)}({ notNull })`
    }
    return buildInlineObjectExpr(stripField(variant, fieldName), { ...ctx, fieldPath: [] }, 6, mode)
  }

  const buildVariantReturn = (dv: DiscriminatedVariant, mode: FieldMode) =>
    `{ ${fieldName}: ${JSON.stringify(dv.typeValue)}, ...${buildVariantInnerExpr(dv, mode)} }`

  const buildCond = (dv: DiscriminatedVariant) => `input?.${fieldName} === ${JSON.stringify(dv.typeValue)}`

  return `
export const ${fullFnName(name)}: DefineFullFn<Types.${getTypeName(name)}> = defineFull<Types.${getTypeName(name)}>(
  (notNull, input) => {
${buildIfChain(discriminatedVariants, buildCond, (dv) => buildVariantReturn(dv, 'full'))}
  }
)

export const ${initFnName(name)}: DefineInitFn<Types.${getTypeName(name)}> = defineInit<Types.${getTypeName(name)}>(
  (input) => {
${buildIfChain(discriminatedVariants, buildCond, (dv) => buildVariantReturn(dv, 'partial'))}
  }
)
`
}

/**
 * Path C: non-discriminated plain union; take the first object variant to generate defaults.
 */
const buildFallbackUnionFn = (name: string, schema: OpenAPISchemaObject, ctx: FieldGenContext): string => {
  const variants = resolveSchemaVariants(schema, ctx.schemas)
  const firstObj = variants.find((v) => v.type === 'object')
  if (!firstObj) return ''
  return buildObjectFn(name, firstObj, ctx)
}

/**
 * Dispatch a union-type schema to one of the three mutually exclusive paths.
 */
const buildUnionDefaultFn = (name: string, schema: OpenAPISchemaObject, ctx: FieldGenContext): string => {
  const rawVariants = schema.oneOf ?? schema.anyOf ?? schema.allOf ?? []
  if (rawVariants.length === 0) return ''

  // Path A: Ref forwarding
  const firstRaw = rawVariants[0]
  if (isRef(firstRaw)) {
    return buildRefForwardFn(name, schema, ctx)
  }

  // Path B: Discriminated union
  const du = tryResolveDiscriminatedUnion(schema, ctx.schemas, ctx.opts?.unionType)
  if (du) {
    return buildDiscriminatedUnionFn(name, du, ctx)
  }

  // Path C: plain union
  return buildFallbackUnionFn(name, schema, ctx)
}

const checkRequriedImports = (ctx: FieldGenContext) => {
  let code = ''
  const { requiredUtils, requiredImports } = ctx
  if (requiredUtils.size) {
    code += `import { ${Array.from(requiredUtils).join(', ')} } from '@hoshiyomi/util/lib/date'
`
  }
  if (requiredImports.has('uuidv4')) {
    code += `import { uuidv4 } from 'uuid'
`
  }
  return code
}

// ─── Plugin entry ──────────────────────────────────────────────────────────────

export const defaultsPlugin = createPlugin((outputDir: string, opts?: DefaultsPluginOpts) => ({
  afterOpenapiParse(document) {
    const schemas: OpenAPISchemas = document.components?.schemas ?? {}
    if (Object.keys(schemas).length === 0) return

    let code = ''

    // Stage 1: collect simple data types (enum, integer alias, pure dict types; functions that return fixed values)
    const dataTypeNames: string[] = []

    const ctx: FieldGenContext = {
      opts,
      dataTypeNames,
      requiredUtils: new Set(),
      requiredImports: new Set(),
      schemas,
    }

    for (const [name, schema] of Object.entries(schemas)) {
      if (!isSchemaObject(schema)) continue
      if (opts?.filter && !opts.filter(name, schema)) continue

      if (schema.enum?.length) {
        const defaultValue = schema.default !== undefined ? schema.default : schema.enum[0]
        code += `\nexport const ${defaultFnName(name)} = (): Types.${getTypeName(name)} => ${JSON.stringify(defaultValue)}\n`
        dataTypeNames.push(name)
      } else if (schema.type === 'number' || schema.type === 'integer') {
        code += `\nexport const ${defaultFnName(name)} = (): number => ${getNonNullExpr(name, schema, ctx)}\n`
        dataTypeNames.push(name)
      } else if (
        schema.type === 'object' &&
        schema.additionalProperties &&
        (!schema.properties || !Object.keys(schema.properties).length)
      ) {
        code += `\nexport const ${defaultFnName(name)} = (): Types.${getTypeName(name)} => ({})\n`
        dataTypeNames.push(name)
      }
    }

    // Stage 2: generate defineFull + defineInit functions for object / union types
    for (const [name, schema] of Object.entries(schemas)) {
      if (!isSchemaObject(schema)) continue
      if (opts?.filter && !opts.filter(name, schema)) continue
      if (dataTypeNames.includes(name)) continue

      if (schema.type === 'object') {
        code += buildObjectFn(name, schema, ctx)
      } else if (schema.oneOf || schema.anyOf || schema.allOf) {
        code += buildUnionDefaultFn(name, schema, ctx)
      }
    }

    code = runtimeHelperCode(checkRequriedImports(ctx)) + code

    writeGeneratedFile(outputDir, 'defaults.ts', code)
  },
}))
