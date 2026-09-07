/// <reference types="node" />

import { createPlugin } from '@alova/wormhole/plugin'
import {
  getRefName,
  getTypeName,
  initFnName,
  isEnum,
  isRef,
  isSchemaObject,
  resolveSchemaVariants,
  tryResolveDiscriminatedUnion,
  type DiscriminatedUnionResult,
  type OpenAPISchema,
  type OpenAPISchemaObject,
  type OpenAPISchemas,
} from '../util/openapi'
import { buildIfChain, writeGeneratedFile } from '../util/codegen'

// ─── Plugin options ───────────────────────────────────────────────────────────

export type PkRule = {
  /** Primary key field name of the associated object, e.g. "id" */
  pk: string
  /** Suffix of the target field, e.g. "_id" */
  suffix: string
  /** Whether it is an array (generates a map expression) */
  isArray?: boolean
  /** Exclude certain fields from PK mapping */
  excludes?: string[]
}

export type ToOpts = {
  /** Filter schemas that do not need to be generated */
  filter?: (name: string) => boolean
  /** PK mapping rules */
  pk?: PkRule[]
  /** Union type discriminator field for associated objects
   * @default 'type'
   */
  unionType?: string
}

// ─── Runtime helper code (injected at the top of generated files) ────────────────

//
// Define a to function that extracts fields from a resp object and outputs a complete req/model object.
//
// Purpose: convert a backend response (resp) type-safely into a request (req) or model type.
// - toFn: extracts all fields from obj (supports nested recursive to)
// - pkFn: PK mapping (e.g. extract id from an associated object: { device_id: obj?.device?.id })
// - fn: initXxx, fills in fields missing from obj (only fields with schema defaults)
//
// The return type is always T (complete type), because to usually extracts from a complete resp object.
//
const RUNTIME_HELPER_CODE = `/* eslint-disable @typescript-eslint/no-explicit-any */
import * as defaults from './defaults'
import type Types from './globals'
import { deepFill } from '@hoshiyomi/util/lib'

type DefineToFn<T> = {
  (obj: any): T
  <O extends object>(obj: any, fnOverride: (obj?: any) => O): O
}

const defineTo = <T extends object>(
  toFn: (obj: any) => Partial<T>,
  pkFn: (obj: any) => Partial<T>,
  fn: (obj?: any) => T,
): DefineToFn<T> => {
  return <O extends object>(obj: any, fnOverride?: (obj?: any) => O): T | O => {
    const result: Partial<T> = obj != null ? toFn(obj) : {}

    if (obj != null) {
      for (const [k, v] of Object.entries(pkFn(obj))) {
        if (v !== undefined) result[k as keyof T] = v as T[keyof T]
      }
    }

    const source = (fnOverride ?? fn)(result);
    deepFill(result, source);

    return result as T | O;
  }
}
`

// ─── Layer 1: PK expression generation ───────────────────────────────────────────

const toFnName = (schemaName: string) => `$to${getTypeName(schemaName)}`

/**
 * Generate a pk mapping expression for the schema's fields based on PK rules
 * e.g. { slave_id: obj?.slave?.id }
 */
const buildPkExpr = (schema: OpenAPISchemaObject, pkRules: PkRule[]): string => {
  const lines: string[] = []

  for (const key of Object.keys(schema.properties ?? {})) {
    const rule = pkRules.find((r) => key.endsWith(r.suffix) && !r.excludes?.includes(key))
    if (!rule) continue

    const sourceProp = key.slice(0, -rule.suffix.length)
    const expr = rule.isArray
      ? `${key}: obj?.${sourceProp}?.map((i: any) => i?.${rule.pk})`
      : `${key}: obj?.${sourceProp}?.${rule.pk}`
    lines.push(expr)
  }

  return lines.length ? `{ ${lines.join(', ')} }` : '{}'
}

// ─── Layer 2: Field access expression generation ─────────────────────────────────

const SRC = 'obj'

/**
 * Determine whether a schema is an inline object with named properties (needs field-by-field expansion)
 */
const isInlineObject = (prop: OpenAPISchema): prop is OpenAPISchemaObject =>
  isSchemaObject(prop) && prop.type === 'object' && !!prop.properties && Object.keys(prop.properties).length > 0

/**
 * Determine whether the schema a $ref points to needs recursive to (object/union type and not enum)
 */
const isToableRef = (refName: string, schemas: OpenAPISchemas): boolean => {
  const target = schemas[refName]
  if (!target || !isSchemaObject(target) || isEnum(target)) return false
  // Pure dict types (e.g. Record<string, any>, only additionalProperties without properties) do not need recursive to
  if (target.type === 'object' && !target.properties) return false
  return !!(target.type === 'object' || target.oneOf || target.anyOf || target.allOf)
}

/**
 * Generate an access expression for a single field
 * @param accessPrefix access path prefix (default "obj?.", pass "obj?.payload?." when nested)
 *
 * Rules:
 * - $ref → recurse into toFn if the target is an object/union, otherwise access directly
 * - array with object $ref items → map + toFn
 * - other → access directly
 */
const buildFieldExpr = (
  fieldName: string,
  schema: OpenAPISchema,
  schemas: OpenAPISchemas,
  accessPrefix = `${SRC}?.`,
): string => {
  const access = `${accessPrefix}${fieldName}`

  if (isRef(schema)) {
    const refName = getRefName(schema.$ref)
    if (isToableRef(refName, schemas)) return `${toFnName(refName)}(${access})`
    return access
  }

  if (!isSchemaObject(schema) || isEnum(schema)) return access

  // array: if item is a deeply-to-able $ref, generate a map expression
  if (schema.type === 'array' && schema.items && isRef(schema.items)) {
    const refName = getRefName(schema.items.$ref)
    if (isToableRef(refName, schemas)) {
      return `${access}?.map((i: any) => ${toFnName(refName)}(i))`
    }
  }

  return access
}

/**
 * Generate an expanded field literal expression for an inline object (has properties)
 * @param parentAccess access path of the parent field, e.g. "obj?.payload"
 */
const buildInlineObjectExpr = (
  parentAccess: string,
  schema: OpenAPISchemaObject,
  schemas: OpenAPISchemas,
  indentSize: number,
): string => {
  const properties = schema.properties ?? {}
  if (Object.keys(properties).length === 0) return parentAccess

  const pad = ' '.repeat(indentSize)
  const innerPad = ' '.repeat(indentSize + 2)

  const lines = Object.entries(properties).map(
    ([key, prop]) => `${innerPad}${key}: ${buildFieldExpr(key, prop, schemas, `${parentAccess}?.`)}`,
  )

  return `{\n${lines.join(',\n')}\n${pad}}`
}

// ─── Layer 3: to function code construction ──────────────────────────────────────

/**
 * Build defineTo(...) function code for a plain object schema
 */
const buildObjectToFn = (
  name: string,
  schema: OpenAPISchemaObject,
  schemas: OpenAPISchemas,
  pkRules: PkRule[],
): string => {
  const fieldLines = Object.entries(schema.properties ?? {}).map(
    ([key, prop]) => `    ${key}: ${buildFieldExpr(key, prop, schemas)}`,
  )

  const typeName = `Types.${getTypeName(name)}`

  return `
export const ${toFnName(name)}: DefineToFn<${typeName}> = defineTo<${typeName}>(
  (obj) => ({
${fieldLines.join(',\n')}
  }),
  (obj) => (${buildPkExpr(schema, pkRules)}),
  defaults.${initFnName(name)}
)
`
}

/**
 * Build defineTo(...) function code with if-chain dispatch for a discriminated union
 * Each variant maps to an if branch; inline object fields expand field by field, the first variant acts as fallback
 */
const buildDiscriminatedUnionToFn = (name: string, du: DiscriminatedUnionResult, schemas: OpenAPISchemas): string => {
  const { fieldName, variants: discriminatedVariants } = du

  const buildCaseReturn = ({ typeValue, schema: variant, refName }: (typeof discriminatedVariants)[number]): string => {
    if (refName) {
      return `{ ${fieldName}: ${JSON.stringify(typeValue)}, ...${toFnName(refName)}(obj) }`
    }

    const otherFields = Object.entries(variant.properties ?? {})
      .filter(([key]) => key !== fieldName)
      .map(([key, prop]) => {
        const expr = isInlineObject(prop)
          ? buildInlineObjectExpr(`${SRC}?.${key}`, prop, schemas, 8)
          : buildFieldExpr(key, prop, schemas)
        return `      ${key}: ${expr}`
      })

    if (otherFields.length === 0) {
      return `{ ${fieldName}: ${JSON.stringify(typeValue)} }`
    }
    return `{
      ${fieldName}: ${JSON.stringify(typeValue)},
${otherFields.join(',\n')}
    }`
  }

  const branches = buildIfChain(
    discriminatedVariants,
    (dv) => `${SRC}?.${fieldName} === ${JSON.stringify(dv.typeValue)}`,
    buildCaseReturn,
  )

  const typeName = `Types.${getTypeName(name)}`

  return `
export const ${toFnName(name)}: DefineToFn<${typeName}> = defineTo<${typeName}>(
  (obj) => {
${branches}
  },
  (obj) => ({}),
  defaults.${initFnName(name)}
)
`
}

// ─── Plugin entry ──────────────────────────────────────────────────────────────

export const toPlugin = createPlugin((outputDir: string, opts?: ToOpts) => ({
  afterOpenapiParse(document) {
    const schemas: OpenAPISchemas = document.components?.schemas ?? {}
    if (Object.keys(schemas).length === 0) return

    const pkRules = opts?.pk ?? []
    let code = RUNTIME_HELPER_CODE

    for (const [name, schema] of Object.entries(schemas)) {
      if (opts?.filter && !opts.filter(name)) continue
      if (!isSchemaObject(schema)) continue

      // Plain object type
      if (schema.type === 'object' && schema.properties) {
        code += buildObjectToFn(name, schema, schemas, pkRules)
        continue
      }

      // oneOf/anyOf/allOf union type
      const du = tryResolveDiscriminatedUnion(schema, schemas, opts?.unionType)
      if (du) {
        // Discriminated union: switch dispatch
        code += buildDiscriminatedUnionToFn(name, du, schemas)
      } else {
        // Plain union type: take the first object variant to generate
        const schemaVariants = resolveSchemaVariants(schema, schemas)
        const firstObj = schemaVariants.find((v) => v.type === 'object' && v.properties)
        if (firstObj) code += buildObjectToFn(name, firstObj, schemas, pkRules)
      }
    }

    writeGeneratedFile(outputDir, 'to.ts', code)
  },
}))
