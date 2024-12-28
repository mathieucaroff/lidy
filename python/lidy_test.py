import re

import specimen

interpolation_regex = r"\$\{([a-zA-Z0-9_]+)\}"


def template_read_entry(input: dict[str, str], key: str):
    value = input.get(key)
    template_value = input.get(f"{key}Template")
    if value is not None and template_value is not None:
        raise ValueError(
            f"Found both '{key}' and '{key}Template'. Only one must be specified."
        )
    if value is None and template_value is None:
        return None
    if value is not None:
        return value

    def handle_replace(match) -> str:
        name = match[2:-1]
        value = input.get(name)
        if value is None:
            raise ValueError(
                f"the template interpolation key '{name}' was not found in the input"
            )
        return value

    return re.sub(interpolation_regex, handle_replace, template_value)


@specimen.run(
    specimen.read_local_file("../../testdata/collection/listOf.spec.yaml"),
    specimen.read_local_file("../../testdata/collection/map.spec.yaml"),
    specimen.read_local_file("../../testdata/collection/mapOf.spec.yaml"),
    specimen.read_local_file("../../testdata/collection/merge.spec.yaml"),
    specimen.read_local_file("../../testdata/collection/min_max_nb.spec.yaml"),
    specimen.read_local_file("../../testdata/collection/tuple.spec.yaml"),
    specimen.read_local_file("../../testdata/combinator/oneOf.spec.yaml"),
    specimen.read_local_file("../../testdata/scalar/in.spec.yaml"),
    specimen.read_local_file("../../testdata/scalar/range.spec.yaml"),
    specimen.read_local_file("../../testdata/scalar/regexp.spec.yaml"),
    specimen.read_local_file("../../testdata/scalarType/scalar.spec.yaml"),
    specimen.read_local_file("../../testdata/schema/document.spec.yaml"),
    specimen.read_local_file("../../testdata/schema/expression.spec.yaml"),
    specimen.read_local_file("../../testdata/schema/mergeChecker.spec.yaml"),
    specimen.read_local_file("../../testdata/schema/regex.spec.yaml"),
    specimen.read_local_file("../../testdata/yaml/yaml.spec.yaml"),
)
def test_handler(
    s, name: str, box="content", errorContains: str | None = None, **kwargs
):
    text = template_read_entry(kwargs, "text")
    if text is None:
        s.fail("The 'text' entry is required")
    expression = template_read_entry(kwargs, "expression")
    schema = template_read_entry(kwargs, "schema")

    if box == "content":
        if expression is not None and schema is not None:
            s.fail("'expression' and 'schema' cannot be specified together")
        if expression is None and schema is None:
            s.fail("one of 'expression' and 'schema' must be specified")
        if expression is not None:
            schema = f"main:\n  {expression.replace('\n', '\n  ')}"
    else:
        word = ""
        if expression is not None:
            word = "expression"
        elif schema is not None:
            word = "schema"
        if word:
            s.fail(f"box {box} should not receive any {word}")

    outcome = ""
    if name.startswith("accept"):
        outcome = "accept"
    elif name.startswith("reject"):
        outcome = "reject"
    if outcome == "":
        s.fail(
            f"the 'name' entry should begin by 'accept' or 'reject', but it is: {name}"
        )

    error = None
    if box == "content":
        if schema == "":
            s.fail("the schema cannot be empty")
        try:
            parser = lidy.make_parser(
                lidy.File(name="<schema>.yaml", content=schema),
                dict(),
            )
        except Exception as e:
            error = e
        if error:
            s.abort(f"error in schema: {error}")
        try:
            parser.parse(lidy.File(name="<content>.yaml", content=text))
        except Exception as e:
            error = e
    else:
        if box == "lidySchemaExpression":
            schema = f"main:\n  {text.replace('\n', '\n  ')}"
        elif box == "lidySchemaDocument":
            schema = text
        elif box == "lidySchemaRegexChecker":
            schema = f"main:\n  _regex: '{text}'"
        else:
            s.fail(f"unknown test box: {box}")
        try:
            lidy.make_parser(
                lidy.File(name="<schema.yaml>", content=schema),
                {},
            )
        except Exception as e:
            error = e

    if outcome == "accept":
        if errorContains:
            s.abort(
                "'errorContains' cannot be specified when the expected outcome is 'accept'"
            )
        if error:
            s.fail(f"error: {error}")
    else:
        if error is None:
            s.fail("no error was found")
        if errorContains:
            if errorContains not in str(err):
                s.fail(f"error message '{err}' does not contain '{errorContains}'")
