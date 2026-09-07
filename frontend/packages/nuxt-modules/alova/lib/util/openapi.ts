import type { OpenAPIV3, OpenAPIV3_1 } from 'openapi-types'

export type OpenAPISchemaObject = OpenAPIV3.SchemaObject | OpenAPIV3_1.SchemaObject
export type OpenAPIReferenceObject = OpenAPIV3.ReferenceObject | OpenAPIV3_1.ReferenceObject

export type OpenAPISchema = OpenAPISchemaObject | OpenAPIReferenceObject
export type OpenAPISchemas = Record<string, OpenAPISchema>

// ─── Basic predicates ─────────────────────────────────────────────────────────

export const isRef = (s: OpenAPISchema): s is OpenAPIReferenceObject => '$ref' in s
export const isSchemaObject = (s: OpenAPISchema): s is OpenAPISchemaObject => !isRef(s)
export const isEnum = (s: OpenAPISchema): boolean => isSchemaObject(s) && 'enum' in s
export const getRefName = (ref: string) => ref.split('/').pop()!

// ─── Naming utilities ─────────────────────────────────────────────────────────

export const getTypeName = (name: string) => {
  const parts = name.split('.')
  return parts
    .map((part, index) => {
      if (index === 0) {
        return part.charAt(0).toUpperCase() + part.slice(1).replaceAll('-', '_')
      }
      return (
        part
          // Handle acronym boundaries: "VOBase" -> "VO_Base"
          .replace(/([A-Z]+)([A-Z][a-z])/g, '$1_$2')
          // Handle camelCase boundaries: "couponVO" -> "coupon_VO"
          .replace(/([a-z\d])([A-Z])/g, '$1_$2')
          .toLowerCase()
          .replaceAll('-', '_')
      )
    })
    .join('_')
}

export const fullFnName = (schemaName: string) => `$full${getTypeName(schemaName)}`
export const initFnName = (schemaName: string) => `$init${getTypeName(schemaName)}`
export const defaultFnName = (schemaName: string) => `$default${getTypeName(schemaName)}`

/** Convert object to compact string, e.g. { min: 1, max: 10 } */
export const toObjStr = (obj: Record<string, unknown>) => {
  const props = Object.entries(obj)
    .filter(([_, v]) => v !== undefined)
    .map(([k, v]) => `${k}: ${JSON.stringify(v)}`)
  return `{ ${props.join(', ')} }`
}

// ─── Discriminated Union ──────────────────────────────────────────────────────

/**
 * A discriminated union variant of oneOf/anyOf/allOf
 * - refName: if the variant is composed of allOf [$ref, {type discriminator}], record the $ref name;
 *   code generation should call fullXxx/initXxx instead of expanding fields inline
 */
export interface DiscriminatedVariant {
  typeValue: string
  schema: OpenAPISchemaObject
  /** $ref schema name contained in allOf; when present, generate a ref function call */
  refName?: string
}

/**
 * Check that all variants have the given field and that the field is a fixed enum value.
 */
const hasEnumFieldInAllVariants = (variants: OpenAPISchemaObject[], fieldName: string): boolean =>
  variants.every((v) => {
    const f = v.properties?.[fieldName]
    return f && isSchemaObject(f) && f.enum?.length === 1
  })

/**
 * Detect the discriminator field name of a discriminated union.
 *
 * Priority:
 *  1. OpenAPI 3.1 discriminator.propertyName (if valid)
 *  2. User-configured preferredField (e.g. the unionType option)
 *  3. Infer from the first variant (find a fixed enum field present in all variants)
 *
 * @param variants list of expanded variants
 * @param preferredField field name to prefer
 * @param parentSchema parent schema, may carry an OpenAPI 3.1 discriminator declaration
 * @returns the discriminator field name, or false if it is not a discriminated union
 */
export const detectDiscriminatorField = (
  variants: OpenAPISchemaObject[],
  preferredField?: string,
  parentSchema?: OpenAPISchemaObject,
): string | false => {
  if (variants.length === 0) return false

  // 1. OpenAPI 3.1 explicit declaration takes priority
  const discField = parentSchema?.discriminator?.propertyName
  if (discField && hasEnumFieldInAllVariants(variants, discField)) {
    return discField
  }

  if (!variants.every((v) => v.type === 'object')) return false

  const firstProps = variants[0].properties ?? {}
  const candidates = Object.entries(firstProps)
    .filter(([, prop]) => isSchemaObject(prop) && prop.enum?.length === 1)
    .map(([key]) => key)

  if (candidates.length === 0) return false

  // 2. User-configured preferred field
  if (preferredField && candidates.includes(preferredField)) {
    if (hasEnumFieldInAllVariants(variants, preferredField)) return preferredField
  }

  // 3. Infer: find the first fixed enum field present in all variants, in candidate order
  for (const field of candidates) {
    if (hasEnumFieldInAllVariants(variants, field)) return field
  }

  return false
}

/**
 * Extract the type value, schema, and optional refName from validated discriminated union variants.
 */
export const extractDiscriminatedVariants = (
  variants: OpenAPISchemaObject[],
  rawVariants?: OpenAPISchema[],
  unionField = 'type',
): DiscriminatedVariant[] => {
  return variants.map((variant, i) => {
    const typeField = variant.properties![unionField] as OpenAPISchemaObject
    const raw = rawVariants?.[i]
    // Check whether the raw variant is an allOf [$ref, ...] structure; extract the $ref name
    const refName =
      raw && isSchemaObject(raw) && raw.allOf
        ? raw.allOf.reduce<string | undefined>(
            (found, part) => found ?? (isRef(part) ? getRefName(part.$ref) : undefined),
            undefined,
          )
        : undefined
    return {
      typeValue: String(typeField.enum![0]),
      schema: variant,
      refName,
    }
  })
}

/**
 * Detect and extract a discriminated union in a single call.
 * Returns the result object on success, null on failure.
 */
export interface DiscriminatedUnionResult {
  fieldName: string
  variants: DiscriminatedVariant[]
}

export const tryResolveDiscriminatedUnion = (
  schema: OpenAPISchemaObject,
  schemas?: OpenAPISchemas,
  preferredField?: string,
): DiscriminatedUnionResult | null => {
  const variants = resolveSchemaVariants(schema, schemas)
  if (variants.length === 0) return null

  const fieldName = detectDiscriminatorField(variants, preferredField, schema)
  if (fieldName === false) return null

  const rawVariants = schema.oneOf ?? schema.anyOf ?? schema.allOf ?? []
  const discriminatedVariants = extractDiscriminatedVariants(variants, rawVariants, fieldName)
  if (!discriminatedVariants[0]) return null

  return { fieldName, variants: discriminatedVariants }
}

/**
 * Remove the given field from an object schema (to exclude the discriminator field when generating variant code).
 */
export const stripField = (schema: OpenAPISchemaObject, fieldName: string): OpenAPISchemaObject => ({
  ...schema,
  required: schema.required?.filter((k) => k !== fieldName) ?? [],
  properties: Object.fromEntries(Object.entries(schema.properties ?? {}).filter(([k]) => k !== fieldName)),
})

/**
 * Flatten allOf, merging all properties to the top level
 */
const flattenAllOf = (schema: OpenAPISchemaObject, schemas?: OpenAPISchemas): OpenAPISchemaObject => {
  if (!schema.allOf) return schema

  const result: OpenAPISchemaObject = { ...schema }
  delete result.allOf

  const properties: Record<string, OpenAPISchema> = {}
  const required: string[] = []

  for (const part of schema.allOf) {
    if (isRef(part) && schemas) {
      const refName = getRefName(part.$ref)
      const refSchema = schemas[refName]
      if (refSchema && isSchemaObject(refSchema) && refSchema.type === 'object' && refSchema.properties) {
        Object.assign(properties, refSchema.properties)
        if (refSchema.required) required.push(...refSchema.required)
      }
      continue
    }

    if (!isSchemaObject(part)) continue

    if (part.type === 'object' && part.properties) {
      Object.assign(properties, part.properties)
      if (part.required) required.push(...part.required)
    }
    if (part.allOf) {
      const nested = flattenAllOf(part, schemas)
      if (nested.properties) Object.assign(properties, nested.properties)
      if (nested.required) required.push(...nested.required)
    }
  }

  result.type = 'object'
  result.properties = properties
  if (required.length > 0) result.required = required
  return result
}

/**
 * Extract all non-Ref variants from a oneOf/anyOf/allOf schema and flatten allOf
 */
export const resolveSchemaVariants = (schema: OpenAPISchemaObject, schemas?: OpenAPISchemas): OpenAPISchemaObject[] => {
  const variants = schema.oneOf ?? schema.anyOf ?? schema.allOf ?? []
  return variants
    .filter((v): v is OpenAPISchemaObject => isSchemaObject(v))
    .map((v) => (v.allOf ? flattenAllOf(v, schemas) : v))
}
