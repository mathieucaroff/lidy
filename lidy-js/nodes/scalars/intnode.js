import { ScalarNode } from "../scalarnode.js"
import { isScalar  } from 'yaml'

export class IntNode extends ScalarNode {
  constructor(ctx, current) {
    super(ctx, 'int', current)
    if (checkCurrent(current)) {
        this.value = current.value
    } else {
      throw ctx.syntaxError(current, `Error: value '${current ? current.value : ""}' is not a number`)
    }
  }

  static checkCurrent(current) {
    return isScalar(current) && (typeof(current.value)) == 'number' && (current.value == Math.floor(current.value))
  }

  static parse(ctx, current) {
    if (checkCurrent(current)) { return new IntNode(ctx, current) }
    return null

  }

}

