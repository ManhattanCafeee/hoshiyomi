/// <reference types="node" />

import { createPlugin } from '@alova/wormhole/plugin'
import fs from 'node:fs'
import path from 'node:path'
import type { OpenAPIV3 } from 'openapi-types'
import { isRef, isSchemaObject, toObjStr, type OpenAPISchemaObject, type OpenAPISchemas } from '../util/openapi'

export type RulesOpts = { filter?: (name: string) => boolean }

const schemaFnName = (schemaName: string) => `rules${schemaName[0]?.toUpperCase()}${schemaName.slice(1)}`

export const valibotToNaiveRulesPlugin = createPlugin((outputDir: string, opts?: RulesOpts) => ({
  afterOpenapiParse(document) {
    const schemas: OpenAPISchemas = document.components?.schemas ?? {}
    if (!schemas) return

    let code = `import { valibotToRules } from '@hoshiyomi/alova/lib/util/valibot-rules'
import type { ObjectEntries, ObjectSchema } from 'valibot'
import * as v from './gen/valibot.gen'

const defineRules =
  <T extends ObjectEntries>(schema: ObjectSchema<T, undefined>) =>
  (overrides?: Required<Parameters<typeof valibotToRules<T>>>['1']['overrides']) =>
    valibotToRules(schema, { overrides })
`

    for (const [name, schema] of Object.entries(schemas)) {
      if (opts?.filter && !opts.filter(name)) continue
      if (!isSchemaObject(schema) || schema.type !== 'object' || !schema.properties) continue

      code += `
export const ${schemaFnName(name)} = defineRules(v.v${name})
`
    }

    const out = path.join(outputDir, 'rules.ts')
    fs.mkdirSync(path.dirname(out), { recursive: true })
    fs.writeFileSync(out, code)
  },
}))

const codeGenOpenApiToNaiveRules = (schema: OpenAPISchemaObject): string => {
  if (!schema.properties) return '{}'

  const lines: string[] = []
  // OpenAPI required is defined at the parent level
  const requiredSet = new Set(schema.required || [])
  for (const [propName, prop] of Object.entries(schema.properties as Record<string, OpenAPISchemaObject>)) {
    // Ignore reference types for now, since internal constraints cannot be determined
    if (isRef(prop)) continue
    const rules: string[] = []
    // Handle Required
    // Corresponds to naiveRulePresets.notUndefined()
    if (requiredSet.has(propName)) {
      rules.push(`naiveRulePresets.notUndefined()`)
    }
    // Handle Nullable
    // OpenAPI defaults to nullable: false. If null is not allowed, add a notNull rule
    // Corresponds to naiveRulePresets.notNull()
    if (!prop.type?.includes('null') && (prop as OpenAPIV3.SchemaObject).nullable !== true) {
      rules.push(`naiveRulePresets.notNull()`)
    }
    // Handle Length
    // Corresponds to naiveRulePresets.length({ min, max })
    if (prop.minLength !== undefined || prop.maxLength !== undefined) {
      rules.push(`naiveRulePresets.length(${toObjStr({ min: prop.minLength, max: prop.maxLength })})`)
    }
    // Handle Number range
    if (prop.minimum !== undefined || prop.maximum !== undefined) {
      rules.push(`naiveRulePresets.number(${toObjStr({ min: prop.minimum, max: prop.maximum })})`)
    }
    // Handle Pattern
    if (prop.pattern !== undefined) {
      rules.push(`naiveRulePresets.pattern(${toObjStr({ pattern: prop.pattern })})`)
    }
    // If the field has rules, add them to the object string
    if (rules.length > 0) {
      lines.push(`${propName}: [${rules.join(', ')}]`)
    }
  }
  // Format the output
  return `{\n${lines.map((l) => `  ${l}`).join(',\n')}\n}`
}

export const naiveRulesPlugin = createPlugin((outputDir: string, opts?: RulesOpts) => ({
  afterOpenapiParse(document) {
    const schemas: OpenAPISchemas = document.components?.schemas ?? {}
    if (!schemas) return

    const presetPath = '@hoshiyomi/naive-ui/utils/rules'
    let code = `import { naiveRulePresets } from '${presetPath}'
`

    for (const [name, schema] of Object.entries(schemas)) {
      if (opts?.filter && !opts.filter(name)) continue

      if (!isSchemaObject(schema) || schema.type !== 'object' || !schema.properties) continue

      const rulesCode = codeGenOpenApiToNaiveRules(schema)

      code += `
export const ${schemaFnName(name)} = ${rulesCode}
`
    }

    const out = path.join(outputDir, 'rules.ts')
    fs.mkdirSync(path.dirname(out), { recursive: true })
    fs.writeFileSync(out, code)
  },
}))
