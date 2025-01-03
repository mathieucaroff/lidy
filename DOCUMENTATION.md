# Lidy Documentation

## Content

- [Lidy Documentation](#lidy-documentation)
  - [Content](#content)
  - [Glossary Notice](#glossary-notice)
  - [Lidy schema syntax](#lidy-schema-syntax)
    - [Lidy identifier](#lidy-identifier)
    - [Lidy expression](#lidy-expression)
    - [Predefined Lidy rules](#predefined-lidy-rules)
    - [Scalar rules](#scalar-rules)
    - [Predefined string checker rules](#predefined-string-checker-rules)
    - [Special checkers](#special-checkers)
      - [any](#any)
      - [anyData](#anydata)
      - [never](#never)
    - [Lidy checker forms](#lidy-checker-forms)
    - [`_regex: ...`, define your own string checker](#_regex--define-your-own-string-checker)
      - [\_regex](#_regex)
    - [`_range: ... <= ... < ...`, bound number values](#_range------bound-number-values)
      - [\_range](#_range)
    - [Hashmap, Dict, Object, !!map, **Map-related checkers**](#hashmap-dict-object-map-map-related-checkers) - [mapChecker](#mapchecker)
      - [`_map`, the structured type](#_map-the-structured-type)
        - [\_map](#_map)
      - [`_mapOf`, the associative container](#_mapof-the-associative-container)
        - [\_mapOf](#_mapof)
        - [\_merge](#_merge)
      - [Using `_map` and `_mapOf` together: Specify a fallback rule](#using-_map-and-_mapof-together-specify-a-fallback-rule)
        - [\_map and \_mapOf together](#_map-and-_mapof-together)
      - [`MapResult`, the common output type for map-related checkers](#mapresult-the-common-output-type-for-map-related-checkers)
        - [MapResult](#mapresult)
    - [Array, Sequence, Slice, Tuple, !!seq, **List-related checkers**](#array-sequence-slice-tuple-seq-list-related-checkers) - [listChecker](#listchecker) - [\_list](#_list) - [\_listFacultative](#_listfacultative) - [\_listOf](#_listof)
    - [OneOf, choose, select, alternaives, options, pick, OR](#oneof-choose-select-alternaives-options-pick-or) - [\_oneOf](#_oneof)
    - [In, exact scalar match in a list of scalars](#in-exact-scalar-match-in-a-list-of-scalars) - [\_in](#_in)
    - [`_nb`, `_min`, `_max`, specify the number of entries in a container](#_nb-_min-_max-specify-the-number-of-entries-in-a-container)
      - [container sizing](#container-sizing)
      - [\_nb](#_nb)
      - [\_min](#_min)
      - [\_max](#_max)
  - [Go API](#go-api)
    - [Invocation in Go, simple use case](#invocation-in-go-simple-use-case)
      - [Create a parser](#create-a-parser)
        - [NewParser](#newparser)
      - [Create a file](#create-a-file)
        - [NewFile](#newfile)
      - [Parse the file](#parse-the-file)
        - [Parse](#parse)
      - [Set the Builder Map](#set-the-builder-map)
        - [With](#with)
      - [Set the parse options](#set-the-parse-options)
        - [Option](#option)
    - [Invocation in Go, advanced use case](#invocation-in-go-advanced-use-case)
      - [Check that a file is Yaml](#check-that-a-file-is-yaml)
        - [Yaml](#yaml)
      - [Check that a file is a Lidy schema](#check-that-a-file-is-a-lidy-schema)
        - [Schema](#schema)
      - [Set the schema target](#set-the-schema-target)
        - [Target](#target)
    - [Builder Map | TODO](#builder-map--todo)
    - [Errors | TODO](#errors--todo)

## Glossary Notice

You might want to have a look at the [glossary](./README.md#glossary).

## Lidy schema syntax

A lidy document is a YAML map of rule identifiers to lidy expressions. The identifier `main` is special: it is the default rule used to parse the YAML content document.

### Lidy identifier

A valid lidy identifier is a dot-separated list of lidy names. A lidy name is a sequence of the characters beginning with one of [`a-zA-Z`], followed by any number of character among [`a-zA-Z0-9_`].

### Lidy expression

A Lidy expression can be a string or a YAML map.

- If it is a string, it must be a valid Lidy identifier. The identifier shall either be one of the [default-lidy-rules](#default-lidy-rules).
- If it is a map, it must be of one of the available [checker forms](#lidy-checker-forms).

### Predefined Lidy rules

The predefined lidy rules are [the scalars](#scalars), [the predefined string checkers](#predefined-string-checkers) and [the special checkers](#special-checkers).

### Scalar rules

Scalars, as defined in the [YAML specification](https://yaml.org/type/#id838503)

- `string`, a YAML string
- `boolean`, a YAML boolean
- `int`, a YAML integer
- `float`, a YAML floating-point value
- `null`, the YAML null value

### Predefined string checker rules

These Lidy checkers match `string` values, and perform extra checks

- `timestamp`, an ISO 8601 datetime
- `binary`, a string consisting only of base64 characters, as defined by [tag:yaml.org,2002:binary](https://yaml.org/type/binary.html)

Also see the [`_regex`](#_regex) keyword.

### Special checkers

###### any

###### anyData

There's only two: `any` and `anyData`. They match any YAML content. They can be defined in Lidy schema as follows:

```yaml
any:
  _oneOf:
    - string
    - boolean
    - int
    - float
    - null
    - { _mapOf: { any: any } }
    - { _listOf: any }
```

Please note that the `anyData` predefined rule stores YAML map contents in the `mapOf` field of `MapResult` instances. The `any` predefined rule ignores the data that it matches. It thus produces a result containing the `null` value instead of the data from the YAML file. If you mean to process the data using a builder, you should use `anyData`. If you don't need to process the data, you can use `any`.

###### never

The `never` predefined rule never matches anything. It is used to produce an error when a rule is applied to a value that should never be encountered.

### Lidy checker forms

Lidy has 5 checker forms.

The scalar checker forms are:

- the regex checker, matching a string
- the in checker, matching an exact scalar

/!\ Scalar checker forms are not to be confused with [lidy expression](DOCUMENTATION.md#lidy-expression).

The container checker forms are:

- the map checker, matching a YAML map
- the seq checker, matching a YAML sequence

Finally, there's also one logical checker form:

- the one-of, selecting the first matching lidy expression

### `_regex: ...`, define your own string checker

##### \_regex

The `_regex` keyword allows you to accept only strings that match the given regex. The regex engine is [re2](https://github.com/google/re2); find the accepted syntax [here](https://github.com/google/re2/wiki/Syntax). Non-strings values are rejected.

Example:

```yaml
url:
  _regex: 'https?://[^/]+\.[a-zA-Z0-9()]{1,6}(/.*)?'
phoneNumber:
  _regex: '\\+?[0-9]+( [0-9]+)*'
```

Note: In [single quoted strings](https://yaml.org/spec/1.1/#id905860) the backslashes `\` are not interpreted. This makes them a good choice of delimiter for regexes.

### `_range: ... <= ... < ...`, bound number values

##### \_range

The `_range` keyword allows you to accept only integers (`int`) or floats (`float`) within certain bounds. The lower and upper bounds are each optional.

Example:

```yaml
diceResult:
  _range: 1 <= int <= 6

random:
  _range: 0 <= float < 1

logisticMapTakesTwoValues:
  _range: 3.0 < float <= 3.45

natural:
  _range: 0 <= int

strictlyNegativeNumber:
  _range: float < 0
```

### Hashmap, Dict, Object, !!map, **Map-related checkers**

###### mapChecker

The `_min`, `_max` and `_nb` keywords apply to the number of entries in the YAML map. See [container sizing](#container-sizing).

#### `_map`, the structured type

###### \_map

Usage:

```yaml
_map: <map of strings to lidy expressions>
_mapFacultative?: <map of strings to lidy expressions>
_min?: <int>
_max?: <int>
_nb?: <int>
```

Example:

```yaml
_map:
  exactPropertyNameA: string
  exactPropertyNameB: int
_mapFacultative:
  exactPropertyNameCforAnOptionalProperty: bool
```

`_map`, defines an association of properties. The keys used in the schema specify are to be exactly matched in the content. The value-side lidy expressions are each used to match the value in the content.

All keys defined in the map will be **required** in the YAML content.

The `_mapFacultative` keyword allows to define optional properties.

```yaml
_map: {}
_mapFacultative:
  name: string
  birthYear: int
```

#### `_mapOf`, the associative container

###### \_mapOf

`_mapOf` defines a map of the given key and value types. The `_mapOf` keyword accepts a map which must contain a single key-value pair. The key and the value of this pair must each be valid checkers. They will each be used to validate all entries of the map.

Usage:

```yaml
_mapOf: <a map with a single entry>
_min?: <int>
_max?: <int>
_nb?: <int>
```

Example:

```yaml
fullname:
  _mapOf:
    string: string
conversionTable:
  _mapOf:
    float: float
sparseArray:
  _mapOf:
    int: any
  _min: 1
```

###### \_merge

Using the `_merge` keyword allows to extend a previously defined map checker.
The extended map checker may itself extend another map checker, but it may not contain a `_mapOf` keyword.

#### Using `_map` and `_mapOf` together: Specify a fallback rule

###### \_map and \_mapOf together

When `_map` and `_mapOf` are used together, the checker first check keys against
the `_map`. If they are found in the `_map`, the checker specified in the `_map` is used for the value. If they are **not** found in the `_map`, then
the two checkers specified in the `_mapOf` are applied, one to the key and one to the value.

Note:

`_mapOf` can be used together with `_map` to produce a `_map` which accepts extra keys. Here's one way to do it:

```yaml
house:
  _map:
    kitchen: room
    bedroom: room
  _mapOf:
    any: any
# The above house is defined as *requering* a kitchen and a bedroom entries,
# both of which must be rooms. However, it may contain any other key, of any
# othe value
```

#### `MapResult`, the common output type for map-related checkers

###### MapResult

`_map` and `_mapOf` both produce a `MapResult` as result output. A `MapResult` is defined in Go as follows:

```go
type MapResult struct {
  Map map[string]Result
  MapOf []KeyValueResult
}
```

```go
type KeyValueResult struct {
	Key   Result
	Value Result
}
```

The MapResult type comes with the helper method `Hashed()`:

```go
func (MapResult) Hashed() (map[string]Result, error)
```

Assuming all keys are strings, `Hashed()` converts the KeyValueResult list into a map of `string`-s to `Result`-s. It errors if any key is not a string.

### Array, Sequence, Slice, Tuple, !!seq, **List-related checkers**

###### listChecker

The `_min`, `_max` and `_nb` keywords apply to the number of entries in the YAML sequence. See [container sizing](#container-sizing). They can only be used if the `_listOf` keyword is used in the definition of the list checker.

A list checker is defined by the use at least one of the lidy keywords `_list`, `_listFacultative` or `_listOf`. List checkers validate that the yaml node is a
sequence, then map the nodes one by one to the checkers specified by `_list`, then those specified by `_listFacultative`, and all remaining nodes are validated by the single checker specified by `_listOf`.

The validated sequence must contain at least one element for each checker of `_list`. All element of the sequence being validated must have a corresponding checker for lidy to accept it. Definition of sequences which can have any number of element require the use of the `_listOf` keyword.

Usage:

```yaml
_list?: <sequence of lidy expressions>
_listFacultative?: <sequence of lidy expressions>
_listOf?: <lidy expression>
_min?: <int>
_max?: <int>
_nb?: <int>
```

Example:

```yaml
_list:
  - cat
  - cat
_listFacultative:
  - dog
  - dog
_listOf:
  - animal
```

###### \_list

###### \_listFacultative

###### \_listOf

### OneOf, choose, select, alternaives, options, pick, OR

###### \_oneOf

`_oneOf` specifies a list of checkers to choose from. The first checker that
matches will be picked and the node will be accepted. If no checker matches, \_oneOf will reject the node.

Usage:

```yaml
_oneOf?: <sequence of lidy expressions>
```

Example:

```yaml
_oneOf:
  - cat
  - dog
  - kangaroo
```

### In, exact scalar match in a list of scalars

###### \_in

`_in` specifies a list of exact **scalar** value that the node may take.

Usage:

```yaml
_oneOf?: <sequence of YAML scalars>
```

Example:

```yaml
_oneOf: [int, float, nullType]
```

### `_nb`, `_min`, `_max`, specify the number of entries in a container

###### container sizing

###### \_nb

The `_nb` keyword allows to specify the exact number of entries that the container must have.

Example (with `_mapOf`):

```yaml
main:
  _mapOf: { string: string }
  _nb: 1
```

In the above example, the yaml map matched by `main` must have a single entry.

###### \_min

Example (with `_map` and `_mapFacultative`):

```yaml
main: person

person:
  _map:
    name: string
  _mapFacultative:
    age: int
    birthYear: int
    wealth: float
  _min: 2
```

In the above example, in the yaml map matched by `person`, at least **one** of the three facultative entries must be provided.

###### \_max

The `_max` keyword allows to specify the maximum number of entries that the container must have.

Example (with `_seq`):

```yaml
main:
  _seq: string
  _max: 1
```

In the above example, the yaml seq matched by `main` must have 0 or 1 entry.

## Go API

_TODO: add descriptions for each possible action_

### Invocation in Go, simple use case

#### Create a parser

###### NewParser

```go
lidy.NewParser("<indicative filename or empty string>", <[]byte YAML schema>))
```

#### Create a file

###### NewFile

```go
file := lidy.NewFile("<indicative filename or empty string>", <[]byte YAML content>)
```

#### Parse the file

###### Parse

```go
result, err := parser.Parse(<lidy File>)
```

#### Set the Builder Map

###### With

```go
builderMap := make(map[string]lidy.Builder)
chainable := parser.With(builderMap)
Expect(chainable).To(Equal(parser))
```

#### Set the parse options

###### Option

```go
chainable := parser.Option(lidy.Option{
  WarnUnusedRule: true,
})
Expect(chainable).To(Equal(parser))
```

### Invocation in Go, advanced use case

#### Check that a file is Yaml

###### Yaml

```go
err := file.Yaml()
```

#### Check that a file is a Lidy schema

###### Schema

```go
err := parser.Schema()
Expect(chainable).To(Equal(parser))
```

#### Set the schema target

###### Target

The default target is `main`.

```go
chainable := parser.Target("tree")
Expect(chainable).To(Equal(parser))
```

### Builder Map | TODO

```go
type Result interface {}

type Builder interface {
  build(input interface{}) (Result, []error)
}
```

The input of the builders can be Lidy Results or other builder's results. The builder can produce any result.

Example:

```go

main: animal

animal:: dog

dog:: string
```

### Errors | TODO
