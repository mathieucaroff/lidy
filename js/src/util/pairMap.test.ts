import * as specimen from "specimen-test"
import * as yaml from "yaml"
import { PairMap } from "./pairMap"

export function pairMapTestHandler(
  s: specimen.SpecimenContext,
  input: Record<string, string>,
) {
  const pairMap = new PairMap<string, string, number>()

  for (const [keyA, keyB, value] of yaml.parse(input.data)) {
    pairMap.set(keyA, keyB, value)
  }

  check(s, pairMap, input)
}

function check(
  s: specimen.SpecimenContext,
  pairMap: PairMap<string, string, number>,
  input: Record<string, string>,
) {
  if (input.property) {
    let value = pairMap[input.property]
    if (value !== yaml.parse(input.is)) {
      s.fail(`${input.property} is ${value} but expected ${input.is}`)
    }
    return
  }

  if (input.callback) {
    let callbackTarget = yaml.parse(input.callback)
    let callbackInfo = {
      args: [] as any[],
    }
    pairMap[input.target]((...args: any[]) => {
      callbackInfo.args.push(args)
    })
    if (callbackInfo.args.length !== callbackTarget.occurrence) {
      s.fail(
        `${input.target} callback is called ${callbackInfo.args.length} times but expected ${callbackTarget.occurrence} times`,
      )
    }
    return
  }

  let result = pairMap[input.target](...yaml.parse(input.with ?? "[]"))
  if (input.is) {
    if (!deepEqual(result, yaml.parse(input.is))) {
      s.fail(
        `${input.target}(...${input["with"]}) is ${result} but expected ${input.is}`,
      )
    }
  } else if (input.isUndefined) {
    if (result !== undefined) {
      s.fail(
        `${input.target}(...${input["with"]}) is ${result} but expected undefined`,
      )
    }
  }
  if (input.then) {
    let then = yaml.parse(input.then)
    check(s, pairMap, then)
  }
}

function deepEqual(a: any, b: any) {
  if (a === b) return true
  if (typeof a !== typeof b) return false
  if (typeof a === "object") {
    return Object.keys(a).every((key) => deepEqual(a[key], b[key]))
  }
  return false
}
