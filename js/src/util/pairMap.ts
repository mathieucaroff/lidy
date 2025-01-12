export class PairMap<TKA, TKB, TV> {
  root: Map<TKA, Map<TKB, TV>>
  constructor() {
    this.root = new Map()
  }
  get size() {
    let total = 0
    for (const map of this.root.values()) {
      total += map.size
    }
    return total
  }
  has(keyA: TKA, keyB: TKB): boolean {
    return this.root.get(keyA)?.has(keyB) ?? false
  }
  get(keyA: TKA, keyB: TKB): TV {
    return this.root.get(keyA)?.get(keyB)
  }
  set(keyA: TKA, keyB: TKB, value: TV) {
    const map = this.root.get(keyA) ?? new Map()
    map.set(keyB, value)
    this.root.set(keyA, map)
  }
  delete(keyA: TKA, keyB: TKB): boolean {
    const map = this.root.get(keyA)
    let result = false
    if (map) {
      result = map.delete(keyB)
      if (map.size === 0) {
        this.root.delete(keyA)
      }
    }
    return result
  }
  clear() {
    this.root.clear()
  }
  keys() {
    return [...this.root.entries()].flatMap(([keyA, map]) =>
      [...map.keys()].map((keyB) => [keyA, keyB]),
    )
  }
  values() {
    return [...this.root.values()].flatMap((map) => [...map.values()])
  }
  entries() {
    const me = this
    function* entries() {
      for (const [keyA, map] of me.root.entries()) {
        yield* [...map.entries()].map(([keyB, value]) => [keyA, keyB, value])
      }
    }
    return entries()
  }
  forEach(callback: (value: TV, keyA: TKA, keyB: TKB) => void) {
    for (const [keyA, map] of this.root.entries()) {
      for (const [keyB, value] of map.entries()) {
        callback(value, keyA, keyB)
      }
    }
  }
}
