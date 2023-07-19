package lidy_test

import (
	"errors"
	"fmt"
	"testing"

	"github.com/ditrit/lidy"
	"github.com/ditrit/specimen/go/specimen"
)

var codeboxSet = specimen.MakeCodeboxSet(map[string]specimen.BoxFunction{
	"base": func(s *specimen.S, input specimen.Dict) {
		target, targetFound := input["target"]
		expression, expressionFound := input["expression"]
		schema, schemaFound := input["schema"]

		if !targetFound {
			target = "content"
		}
		if expressionFound && schemaFound {
			s.Fail("'expression' and 'schema' cannot be specified together")
		}
		if !expressionFound && !schemaFound {
			s.Fail("One of 'expression' and 'schema' must be specified")
		}
		if expressionFound {
			expressionSchema := fmt.Sprintf("main:\n  %s", expression.(string))

			schema = expressionSchema
		}

		outcome := input["outcome"].(string)
		if outcome != "accept" && outcome != "reject" {
			s.Fail(fmt.Sprintf("Encountered an unhandeled 'outcome' value: %s", outcome))
		}
		text := input["text"].(string)

		// // // // // // // // // // // // // // // // // // // // //
		var err error
		if target == "content" {
			parser := lidy.Parser{
				File: lidy.File{Name: "<schema>.yaml", Content: []byte(schema.(string))},
			}
			_, err = parser.Parse(
				lidy.File{Name: "<content>.yaml", Content: []byte(text)},
			)
		} else if target == "lidySchemaExpression" {
			parser := lidy.Parser{
				File: lidy.File{Name: "<schema>.yaml", Content: []byte(fmt.Sprintf("main:\n  %s", text))},
			}
			parser.Check()
		}
		// // // // // // // // // // // // // // // // // // // // //

		if outcome == "accept" {
			if errors.Unwrap(err) != nil {
				s.Fail("some error(s)")
			}
		} else {
			if errors.Unwrap(err) == nil {
				s.Fail("no error")
			}
		}
	},
})

func TestLidy(t *testing.T) {
	specimen.Run(
		t,
		codeboxSet,
		[]specimen.File{
			specimen.ReadLocalFile("../testdata/collection/listOf.spec.yaml"),
			specimen.ReadLocalFile("../testdata/collection/map.spec.yaml"),
			specimen.ReadLocalFile("../testdata/collection/mapOf.spec.yaml"),
			specimen.ReadLocalFile("../testdata/collection/merge.spec.yaml"),
			specimen.ReadLocalFile("../testdata/collection/min_max_nb.spec.yaml"),
			specimen.ReadLocalFile("../testdata/collection/tuple.spec.yaml"),
			specimen.ReadLocalFile("../testdata/combinator/oneOf.spec.yaml"),
			specimen.ReadLocalFile("../testdata/scalar/in.spec.yaml"),
			specimen.ReadLocalFile("../testdata/scalar/regexp.spec.yaml"),
			specimen.ReadLocalFile("../testdata/scalarType/scalar.spec.yaml"),
			specimen.ReadLocalFile("../testdata/schema/document.spec.yaml"),
			specimen.ReadLocalFile("../testdata/schema/expression.spec.yaml"),
			specimen.ReadLocalFile("../testdata/schema/mergeChecker.spec.yaml"),
			specimen.ReadLocalFile("../testdata/schema/regex.spec.yaml"),
			specimen.ReadLocalFile("../testdata/yaml/yaml.spec.yaml"),
		},
	)
}
