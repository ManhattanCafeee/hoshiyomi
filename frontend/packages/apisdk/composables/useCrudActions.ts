interface AlovaMethod<T> {
  send(): Promise<T>
}

export function useCrudActions<T extends { id?: number }>(
  items: Ref<T[]>,
  methods: {
    create?: (config: { data: Partial<T> }) => AlovaMethod<T>
    update?: (config: { pathParams: { id: number }; data: Partial<T> }) => AlovaMethod<T>
    remove?: (config: { pathParams: { id: number } }) => AlovaMethod<void>
  },
) {
  const createItem = async (data: Partial<T>): Promise<T> => {
    if (!methods.create) throw new Error('create not configured')
    const resp = await methods.create({ data }).send()
    if (resp != null) items.value.unshift(resp as T)
    return resp as T
  }

  const updateItem = async (id: number | undefined, data: Partial<T>): Promise<T> => {
    if (id == null) throw new Error('updateItem: id is required')
    if (!methods.update) throw new Error('update not configured')
    const resp = await methods.update({ pathParams: { id }, data }).send()
    const idx = items.value.findIndex((x) => x.id != null && x.id === id)
    if (idx !== -1) {
      items.value[idx] = (resp ?? { ...items.value[idx], ...data }) as T
    }
    return resp as T
  }

  const deleteItem = async (id: number | undefined): Promise<void> => {
    if (id == null) throw new Error('deleteItem: id is required')
    if (!methods.remove) throw new Error('remove not configured')
    await methods.remove({ pathParams: { id } }).send()
    const idx = items.value.findIndex((x) => x.id != null && x.id === id)
    if (idx !== -1) items.value.splice(idx, 1)
  }

  return { createItem, updateItem, deleteItem }
}
