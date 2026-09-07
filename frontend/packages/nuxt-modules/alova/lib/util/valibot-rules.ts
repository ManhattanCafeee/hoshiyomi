import type { FormRules } from 'naive-ui'
import { safeParse, type ObjectEntries, type ObjectSchema } from 'valibot'

type FormItemRule = FormRules[keyof FormRules]

export function valibotToRules<T extends ObjectEntries>(
  schema: ObjectSchema<T, undefined>,
  options?: {
    overrides?: Record<keyof T, FormItemRule | ((rule: FormItemRule) => FormItemRule)>
  },
): FormRules {
  const rules: FormRules = {}

  for (const [key, fieldSchema] of Object.entries(schema.entries)) {
    const isOptional = fieldSchema.type === 'optional' || fieldSchema.type === 'null' || fieldSchema.type === 'nullable'

    const rule: FormItemRule = {
      required: !isOptional,
      trigger: ['input', 'blur'],
      validator: (_rule: FormItemRule, value: unknown) => {
        const result = safeParse(fieldSchema, value)

        if (result.success) return true

        return new Error(result.issues[0].message)
      },
    }

    const override = options?.overrides?.[key]

    rules[key] = override instanceof Function ? override(rule) : { ...rule, ...override }
  }

  return rules
}
