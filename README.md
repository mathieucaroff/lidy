<p align="center">
  <img src="https://raw.githubusercontent.com/ditrit/lidy/master/asset/img/lidy-logo.svg" width="250px">
</p>
<p align="center">
  <a href="https://goreportcard.com/report/github.com/ditrit/lidy"><img src="https://goreportcard.com/badge/github.com/ditrit/lidy?style=flat-square" alt="GoReport"></a>
  <a href="http://godoc.org/github.com/ditrit/lidy"><img src="https://img.shields.io/badge/go-documentation-blue.svg?style=flat-square" alt="GoDoc"></a>
  <a href="https://github.com/ditrit/lidy/blob/master/LICENSE"><img src="https://img.shields.io/github/license/ditrit/lidy?style=flat-square" alt="License"></a>
</p>
<hr>
<h3 align="center">The YAML-YAML type-oriented schema validation tool</h3>
<hr>

Lidy is:

- The Lidy schema-type language, a YAML language to specify how to check YAML files
- An engine to run the Lidy schema on YAML files
- A rich deserialisation tool (if you want it to)

Currently, the Lidy project is available in two languages:

- Javascript, in the repository [**lidy-js**](https://github.com/ditrit/lidy-js)
- Golang, in the repository **lidy-go** (this repository)

Both implementations fully adhere to the specification described in the documentation.

However, the way to invoke the parsers currently differs between the two implementations:

- lidy-go uses the concept of builder (see the specific documentation)
- lidy-js uses a similar interface (and in particular the principle of listeners) to that offered by the [antlr](https://www.antlr.org/) tool.

## Content

- [Content](#content)
- [JSON schema](#json-schema)
  - [About lidy's refs](#about-lidys-refs)
- [Example](#example)
- [Alternatives: YAML Schema validators](#alternatives-yaml-schema-validators)
- [Using Regex](#using-regex)
- [Documentation](#documentation)
- [Short reference](#short-reference)
  - [Glossary](#glossary)
  - [Lidy expression](#lidy-expression)
  - [Predefined lidy rules](#predefined-lidy-rules)
    - [Scalar types](#scalar-types)
    - [Predefined string checker rules](#predefined-string-checker-rules)
    - [`any` and `anyData` - Any yaml content](#any-and-anydata---any-yaml-content)
  - [Container checkers](#container-checkers)
    - [Map checkers](#map-checkers)
    - [List checkers](#list-checkers)
  - [Composite checkers](#composite-checkers)
  - [Container checkers](#container-checkers-1)
  - [Scalar checkers](#scalar-checkers)
- [Not yet in Lidy](#not-yet-in-lidy)
  - [Functional types (aka type parameter aka template types)](#functional-types-aka-type-parameter-aka-template-types)
- [Contributing](#contributing)
  - [Developing](#developing)

## JSON schema

What's the point of Lidy, when there's already JSON schema?

- **YAML**: Lidy targets YAML rather than JSON. Of course, it _does_ work with JSON perfectly fine.
- **Refs**: In Lidy, refs are first class citizens, they are just like in programming languages: `<name>`, as opposed to JSON Schema's heavy `{ ref: "#/<name>" }`, see below.
- **Line numbers**: Lidy is meant to _assist_ your users with writing YAML: Lidy provides the line numbers at which the checking failed.
- **Algebriac data types**: Lidy schema are similar to Algebriac data types. They have union types (`_oneOf`), positional product types (`_list`), named product types (`_map`), and combined types (`_merge`). (N.B. parameterized types aren't yet there, but they are on our short list).
- **Rich deserialisation**: Lidy provides support for rich deserialisation. It is a core use-case. This includes access to the source line numbers.
- **Custom checkers**: Writing a custom value checker is also a core use-case. Actually, it's just as easy as writing a type deserialiser since Lidy handles the two through the same interface.
- **Language Recognition**: Lidy is a lexical and grammar parser that can build and walk parse trees. It is completely equivalent to a tool like **antlr**, but for YAML-based DSLs, given that **antlr** is unable to parse languages built on top of YAML.

### About lidy's refs

Where you used to write `"a": { "ref": "#/$def/b" }`, in JSON schema, you now write `"a": "b"`, which is much shorter. Lidy does not support accessing non-root nodes. All nodes that must be referred to must be at the root of the Lidy schema.

Note: Lidy does not yet support remote references.

## Example

`main.go`

```go
package main

import (
	"fmt"

	"github.com/ditrit/lidy"
)

func main() {
	result, errorList := lidy.NewParser(
		"treeDefinition.yaml",
		[]byte(`
main: tree

tree:
  _map:
    name: string
    children:
      _listOf: tree
`),
	).Parse(
		lidy.NewFile(
			"treeContent.yaml",
			[]byte(`
name: root
children:
  - name: leafA
    children: []
  - name: branchB
    children:
    - name: leafC
      children: []
  - name: leafD
    children: []
`),
		),
	)

	if len(errorList) > 0 {
		panic(errorList[0])
	}

	mapResult := result.(lidy.MapResult)

	fmt.Println(mapResult)
}
```

## Alternatives: YAML Schema validators

Here's a list of schema validators we could find:

- Kwalify, [[source (mirror?)]](https://github.com/sunaku/kwalify/), [[website]](http://www.kuwata-lab.com/kwalify/) (Ruby and Java, v0.7.2, 2010-07-18)
- pykwalify [[source]](https://github.com/Grokzen/pykwalify), [[documentation]](https://pykwalify.readthedocs.io/en/master) (Python, v1.7.0, 2018-08-03)
- Rx [[source]](https://github.com/rjbs/Rx), [[website]](http://rx.codesimply.com/) (Js, Perl, PHP, Python, Ruby, v0.200006, 2014-05-21)

Also see the [dedicated page on JSON Schema Everywhere](https://json-schema-everywhere.github.io/yaml).

And a few more project(s):

- Azuki [[source]](https://github.com/guoyk93/azuki), just a Map evaluation tool (Java)

None has the feature-set of Lidy, nor its type-oriented approach.

## Using Regex

If you need a regex to match a well-known format, think of going shopping for it before you start writing it. Ressources: [RgxDB](https://rgxdb.com), [Regex101](https://regex101.com/library?search=&orderBy=MOST_POINTS).

## Documentation

See [DOCUMENTATION.md](DOCUMENTATION.md)

## Short reference

### Glossary

<dl>
  <dt>Expression</dt>
  <dd>A lidy expression specifies a way to check a yaml value</dd>
  <dt>Rule</dt>
  <dd>A user rule declaration gives a rule name to an expression</dd>
  <dt>Builder</dt>
  <dd>A builder is a user-provided function which can process the data read by a rule, and change its content, or produce an error</dd>
  <dt>Scalar type</dt>
  <dd>Scalar types are predefined lidy rules which allow to check for a given scalar type, i.e. a type that is not a container</dd>
  <dt>Container checker</dt>
  <dd>A container checker allows to create a lidy expression for a YAML map or a YAML sequence matching certain types</dd>
</dl>

### Lidy expression

A lidy expression is either:

- The name of a predefined lidy rule
- The name of a lidy rule defined in the same document
- A YAML map which associates one or more lidy keywords to its YAML argument. See [Lidy checker forms](DOCUMENTATION.md#lidy-checker-forms).
  - Note: Not all keyword combinations are valid

Also see [lidy expression](DOCUMENTATION.md#lidy-expression).

### Predefined lidy rules

Also see [predefined lidy rules](DOCUMENTATION.md#predefined-lidy-rules).

#### Scalar types

- `boolean`
- `float`
- `int` -- integer
- `string`
- `nullType` -- null

Also see [Scalar rules](DOCUMENTATION.md#scalar-rules).

#### Predefined string checker rules

- `timestamp` -- ISO 8601 datetime
- `binary` -- a base64 encoded binary blob, with space characters allowed

Also see [Predefined string checker rules](DOCUMENTATION#predefined-string-checker-rules).

#### `any` and `anyData` - Any yaml content

- `any`, `anyData` -- any yaml structure. See [any](DOCUMENTATION.md#any)

The difference between `any` and `anyData` is in how they process the yaml structure that they match. `any` simply ignores the data and produces a result whose data is `null` (`nil` or `null` or `None`), while `anyData` processes the yaml structure it matches into a tree of `Result` elements.

#### `never`

- The `never` predefined rule never matches anything. It is used to produce an error when a rule is applied to a value that should never be encountered.

### Container checkers

#### Map checkers

- [`_map`](DOCUMENTATION.md#_map) -- followed by a map of exact keys to lidy expressions
- [`_mapOf`](DOCUMENTATION.md#_mapOf) -- Example: `_mapOf: { string: int }`
- [`_merge`](DOCUMENTATION.md#_merge) -- create a map checker merging the keys of the listed map checkers
- [`_mapFacultative`](DOCUMENTATION.md#_mapOptional) -- like `_map`, but the specified entries aren't mendatory

#### List checkers

- [`_list`](DOCUMENTATION.md#_list) -- (the equivalent of `_map` but for sequences. It could have been named `_seq`)
- [`_listOf`](DOCUMENTATION.md#_listOf)
- [`_listFacultative`](DOCUMENTATION.md#_seqOptional)

### Composite checkers

- [`_oneOf`](DOCUMENTATION.md#_oneOf) -- accept a list of lidy expressions and select the first that matches, or fail

### Container checkers

- [`_nb`](DOCUMENTATION.md#_nb) -- the container must exactly have the specified number of entries
- [`_max`](DOCUMENTATION.md#_max) -- the container must have at most the specified number of entries
- [`_min`](DOCUMENTATION.md#_min) -- the container must have at least the specified number of entries

### Scalar checkers

- [`_regex`](DOCUMENTATION.md#_regex) -- applies only to strings. Accepted syntax at https://github.com/google/re2/wiki/Syntax
- [`_in`](DOCUMENTATION.md#_in) -- an exact enumeration of terminal YAML values the value must be part of
- [`_range`](DOCUMENTATION.md#_range) -- a range over integers or floats with inclusive or exclusive boundaries

## Not yet in Lidy

### Functional types (aka type parameter aka template types)

Declare a parameter type name:

```yaml
<ContentType>: []
# the <> are forbidden in lidy identifiers. This form is detected as a
# parameter type name declaration
```

Declare a functional type:

```yaml
tree<ContentType>:
  _map:
    name: string
    children: treeChildren
# treeChildren requires a parameter: `treeChildren<ContentType>`
# but lidy is smart enougth to pass it from the parent automatically, since they
# uses the same type name

treeChildren<ContentType>:
  _listOf: treeOrContent

treeOrContent<ContentType>:
  _oneOf:
    - tree
    - ContentType
```

Refer to the functional type:

```yaml
main: tree<boolean>
```

## Contributing

If you have any idea that you'd like to see added to Lidy, please create an issue in the [issue tracker](https://github.com/ditrit/lidy/issues) to share your feature request with us (remember to search-check for duplicate issues first).

You're also welcome to report bugs, ask questions or use the issue tracker as you see fit. We try to be welcoming.

### Developing

Cloning:

```sh
git clone https://github.com/ditrit/lidy
cd lidy
```

Running Lidy's go tests:

```sh
cd go/lidy; go test
```
