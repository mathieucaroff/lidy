import { readFileSync } from "fs"
import * as path from "path"
import * as specimen from "specimen-test"
import * as lidy from "."

var interpolationRegex = /\$\{([a-zA-Z0-9_]+)\}/g

function templateReadEntry(
  input: Record<string, string>,
  key: string,
): string | undefined {
  let value = input[key]
  let templateValue = input[`${key}Template`]
  if (value !== undefined && templateValue !== undefined) {
    throw new Error(
      `Found both '${key}' and '${key}Template'. Only on must be specified.`,
    )
  }
  if (value === undefined && templateValue === undefined) {
    return undefined
  }
  if (value !== undefined) {
    return value
  }

  return templateValue.replace(interpolationRegex, (match) => {
    let name = match.slice(2, -1)
    let value = input[name]
    if (value === undefined) {
      throw new Error(
        `the template interpolation key '${name} was not found in the input`,
      )
    }
    return value
  })
}

function specimenHandler(
  s: specimen.SpecimenContext,
  input: Record<string, string>,
) {
  // Target
  const box = input.box ?? "content"

  // Unpacking the input data

  // Text
  const text = templateReadEntry(input, "text")
  if (text === undefined) {
    s.fail("The 'text' entry is required")
  }

  // Expression and Schema
  const expression = templateReadEntry(input, "expression")
  let schema = templateReadEntry(input, "schema")

  if (box === "content") {
    if (expression !== undefined && schema !== undefined) {
      s.fail("'expression' and 'schema' cannot be specified together")
    }
    if (expression === undefined && schema === undefined) {
      s.fail("one of 'expression' and 'schema' must be specified")
    }
    if (expression !== undefined) {
      schema = `main:\n  ${expression.replace(/\n/g, "\n  ")}`
    }
  } else {
    let word = ""
    if (expression) {
      word = "expression"
    } else if (schema) {
      word = "schema"
    }
    if (word !== "") {
      s.fail(`box ${box} should not receive any ${word}`)
    }
  }

  // Name and Outcome
  const name = input.name
  let outcome = ""
  if (name.startsWith("accept")) {
    outcome = "accept"
  } else if (name.startsWith("reject")) {
    outcome = "reject"
  }
  if (outcome === "") {
    s.fail(
      `the 'name' entry should begin by 'accept' or 'reject', but it is: ${name}`,
    )
  }

  // // // // // // // // // // // // // // // // // // // //
  let error: Error | undefined
  let parser: lidy.Parser | undefined
  if (box === "content") {
    if (!schema) {
      s.fail("the schema cannot be empty")
    }
    try {
      parser = lidy.makeParser({ name: "<schema>.yaml", content: schema }, {})
    } catch (e) {
      let message = `error in schema: ${e.message}`
      s.abort(message)
    }
    try {
      parser.parse({ name: "<content>.yaml", content: text })
    } catch (e) {
      error = e
    }
  } else {
    schema = {
      lidySchemaExpression: () => `main:\n  ${text.replace(/\n/g, "\n  ")}`,
      lidySchemaDocument: () => text,
      lidySchemaRegexChecker: () => `main:\n  _regex: '${text}'`,
    }[box]?.()
    if (schema === undefined) {
      s.fail(`unknown test box: ${box}`)
    }
    try {
      lidy.makeParser({ name: "<schema>.yaml", content: schema }, {})
    } catch (e) {
      error = e
    }
  }

  // // // // // // // // // // // // // // // // // // // //

  const errorContains = input.errorContains
  if (outcome === "accept") {
    if (errorContains !== undefined) {
      s.fail(
        "'errorContains' cannot be specified when the expected outcome is 'accept'",
      )
    }
    if (error !== undefined) {
      s.fail(`error: ${error}`)
    }
  } else {
    if (error === undefined) {
      s.fail(`no error was found (${input.name})`)
    }
    if (errorContains !== undefined) {
      if (!error.message.includes(errorContains)) {
        s.fail(
          `error message '${error.message}' does not contain '${errorContains}'`,
        )
      }
    }
  }
}

function readLocalFile(filepath: string): specimen.File {
  const file = path.join(__dirname, filepath)
  return {
    path: file,
    content: readFileSync(file, "utf-8"),
  }
}

specimen.run(specimenHandler, [
  readLocalFile("../../testdata/collection/listOf.spec.yaml"),
  readLocalFile("../../testdata/collection/map.spec.yaml"),
  readLocalFile("../../testdata/collection/mapOf.spec.yaml"),
  readLocalFile("../../testdata/collection/merge.spec.yaml"),
  readLocalFile("../../testdata/collection/min_max_nb.spec.yaml"),
  readLocalFile("../../testdata/collection/tuple.spec.yaml"),
  readLocalFile("../../testdata/combinator/oneOf.spec.yaml"),
  readLocalFile("../../testdata/scalar/in.spec.yaml"),
  readLocalFile("../../testdata/scalar/range.spec.yaml"),
  readLocalFile("../../testdata/scalar/regexp.spec.yaml"),
  readLocalFile("../../testdata/scalarType/scalar.spec.yaml"),
  readLocalFile("../../testdata/schema/document.spec.yaml"),
  readLocalFile("../../testdata/schema/expression.spec.yaml"),
  readLocalFile("../../testdata/schema/mergeChecker.spec.yaml"),
  readLocalFile("../../testdata/schema/regex.spec.yaml"),
  readLocalFile("../../testdata/yaml/yaml.spec.yaml"),
])
