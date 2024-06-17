package lidy_test

import (
	"fmt"
	"regexp"
	"strings"
	"testing"

	"github.com/ditrit/lidy/go/lidy"
	"github.com/ditrit/specimen/go/specimen"
)

var interpolationRegex = regexp.MustCompile(`\$\{([a-zA-Z0-9_]+)\}`)

func templateReadEntry(input specimen.Dict, key string) (string, bool) {
	value, found := input[key]
	templateValue, templateFound := input[fmt.Sprintf("%sTemplate", key)]
	if found && templateFound {
		panic(fmt.Sprintf("Found both '%s' and '%sTemplate'. Only one must be specified.", key, key))
	}
	if !found && !templateFound {
		return "", false
	}
	if found {
		return value, true
	}

	// The template was found. We need to parse it and replace the placeholders.
	resultValue := interpolationRegex.ReplaceAllStringFunc(templateValue, func(match string) string {
		key := match[2 : len(match)-1]
		value, found := input[key]
		if !found {
			panic(fmt.Sprintf("The template interpolation key '%s' was not found in the input", key))
		}
		return value
	})
	return resultValue, true
}

func specimenHandler(s *specimen.S, input specimen.Dict) {
	// Target
	box, boxFound := input["box"]
	if !boxFound {
		box = "content"
	}

	// Unpacking the input data

	// Text
	text, textFound := templateReadEntry(input, "text")
	if !textFound {
		s.Fail("The 'text' field is required")
	}
	// Expression and Schema
	expression, expressionFound := templateReadEntry(input, "expression")
	schema, schemaFound := templateReadEntry(input, "schema")

	if box == "content" {
		if expressionFound && schemaFound {
			s.Fail("'expression' and 'schema' cannot be specified together")
		}
		if !expressionFound && !schemaFound {
			s.Fail("One of 'expression' and 'schema' must be specified")
		}
		if expressionFound {
			schema = fmt.Sprintf("main:\n  %s", strings.ReplaceAll(expression, "\n", "\n  "))
		}
	} else {
		word := ""
		if expressionFound {
			word = "expression"
		}
		if schemaFound {
			word = "schema"
		}
		if word != "" {
			s.Fail(fmt.Sprintf("box %s should not receive any %s", box, word))
		}
	}

	// Name
	name := input["name"]
	outcome := ""
	if strings.HasPrefix(name, "accept") {
		outcome = "accept"
	} else if strings.HasPrefix(name, "reject") {
		outcome = "reject"
	}
	if outcome == "" {
		s.Fail(fmt.Sprintf("The name should begin by 'accept' or 'reject', but it is: %s", name))
	}

	// // // // // // // // // // // // // // // // // // // // //
	var err error
	if box == "content" {
		if schema == "" {
			s.Fail("The schema cannot be empty")
		}
		parser, parserError := lidy.MakeParser(
			lidy.File{Name: "<schema>.yaml", Content: []byte(schema)},
			map[string]lidy.Builder{},
		)
		if parserError != nil {
			err = parserError
			s.Abort(fmt.Sprintf("error in schema: %s", err.Error()))
		}
		_, contentError := parser.Parse(
			lidy.File{Name: "<content>.yaml", Content: []byte(text)},
		)
		err = contentError
	} else {
		switch box {
		case "lidySchemaExpression":
			schema = fmt.Sprintf("main:\n  %s", strings.ReplaceAll(text, "\n", "\n  "))
		case "lidySchemaDocument":
			schema = text
		case "lidySchemaRegexChecker":
			schema = fmt.Sprintf("main:\n  _regex: '%s'", text)
		default:
			s.Fail(fmt.Sprintf("Unknown test box: %s", box))
			return
		}
		_, parserError := lidy.MakeParser(
			lidy.File{Name: "<schema>.yaml", Content: []byte(schema)},
			map[string]lidy.Builder{},
		)
		err = parserError
	}

	// // // // // // // // // // // // // // // // // // // // //

	errorContains, errorContainsFound := input["errorContains"]
	if outcome == "accept" {
		if errorContainsFound {
			s.Abort("'errorContains' cannot be specified when the expected outcome is 'accept'")
		}
		if err != nil {
			s.Fail(fmt.Sprintf("error: %s", err.Error()))
		}
	} else {
		if err == nil {
			s.Fail("no error was found")
		}
		if errorContainsFound {
			if !strings.Contains(err.Error(), errorContains) {
				s.Fail(fmt.Sprintf("error message '%s' does not contain '%s'", err.Error(), errorContains))
			}
		}
	}
}

func TestLidy(t *testing.T) {
	specimen.Run(
		t,
		specimenHandler,
		[]specimen.File{
			specimen.ReadLocalFile("../../testdata/collection/listOf.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/collection/map.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/collection/mapOf.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/collection/merge.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/collection/min_max_nb.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/collection/tuple.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/combinator/oneOf.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/functional/binaryTree.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/functional/either.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/functional/listOfList.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/functional/oneOrSeveral.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/functional/treeChildren.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/functional/tree.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/scalar/in.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/scalar/range.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/scalar/regexp.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/scalarType/scalar.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/schema/document.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/schema/expression.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/schema/mergeChecker.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/schema/regex.spec.yaml"),
			specimen.ReadLocalFile("../../testdata/yaml/yaml.spec.yaml"),
		},
	)
}
