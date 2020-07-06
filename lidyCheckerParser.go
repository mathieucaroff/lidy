package lidy

import (
	"fmt"
	"regexp"

	"github.com/ditrit/lidy/errorlist"
	"gopkg.in/yaml.v3"
)

// lidyCheckerParser.go
//
// Schema parsing for lidy checkers and checkerForms.

type tChecker func(sp tSchemaParser, node yaml.Node, formMap tFormMap) (tExpression, []error)

type tFormMap map[string]yaml.Node

func mapChecker(sp tSchemaParser, node yaml.Node, formMap tFormMap) (tExpression, []error) {
	errList := errorlist.List{}

	form, erl := mapForm(sp, node, formMap)
	errList.Push(erl)

	sizing, erl := sizingChecker(sp, node, formMap)
	errList.Push(erl)

	return tMap{
		form,
		sizing,
	}, errList.ConcatError()
}

func seqChecker(sp tSchemaParser, node yaml.Node, formMap tFormMap) (tExpression, []error) {
	errList := errorlist.List{}

	form, err := seqForm(sp, node, formMap)
	errList.Push(err)

	sizing, err := sizingChecker(sp, node, formMap)
	errList.Push(err)

	return tSeq{
		form,
		sizing,
	}, errList.ConcatError()
}

func mapParameter(sp tSchemaParser, node yaml.Node, errList *errorlist.List) map[string]tExpression {
	if node.Kind != yaml.MappingNode {
		errList.Push(sp.schemaError(node, "a YAML map"))
		return nil
	}

	result := map[string]tExpression{}

	for k := 0; k+1 < len(node.Content); k += 2 {
		key := *node.Content[k]
		value := *node.Content[k+1]

		if key.Tag != "!!str" {
			errList.Push(sp.schemaError(key, "only string keys"))
			continue
		}

		expression, erl := sp.expression(value)
		errList.Push(erl)

		if len(erl) == 0 {
			result[key.Value] = expression
		}
	}

	return result
}

func checkMergeable(sp tSchemaParser, node yaml.Node, expression tExpression) (tMergeableExpression, []error) {
	if mergeable, ok := expression.(tMap); ok {
		return mergeable, nil
	}

	if oneOf, ok := expression.(tOneOf); ok {
		errList := errorlist.List{}
		for _, option := range oneOf.optionList {
			_, erl := checkMergeable(sp, node, option)
			errList.Push(erl)
		}

		return oneOf, errList.ConcatError()
	}

	if rule, ok := expression.(*tRule); ok {
		return checkMergeable(sp, node, rule.expression)
	}

	return nil, sp.schemaError(node, "a mergeable expression")
	// TODO
	// the value of `node` is imprecise. The exact link to the node, or at least to it's position
	// should be kept in all the checker types.
	// the returned errors should be wrapped too.

	// TODO
	// checkMergeable relies on rule.expression being available. This may not be
	// the case.
}

func mapForm(sp tSchemaParser, node yaml.Node, formMap tFormMap) (tMapForm, []error) {
	errList := errorlist.List{}

	propertyMapNode, _map := formMap["_map"]
	optionalMapNode, _optional := formMap["_optional"]
	mapOfNode, _mapOf := formMap["_mapOf"]
	mergeNode, _merge := formMap["_merge"]

	propertyMap := map[string]tExpression{}
	optionalMap := map[string]tExpression{}
	mapOf := tKeyValueExpression{}
	mergeList := []tMergeableExpression{}

	if _map {
		propertyMap = mapParameter(sp, propertyMapNode, &errList)
	}

	if _optional {
		optionalMap = mapParameter(sp, optionalMapNode, &errList)
	}

	if _mapOf {
		if mapOfNode.Kind != yaml.MappingNode || len(mapOfNode.Content) != 2 {
			errList.Push(sp.schemaError(mapOfNode, "a YAML map, with a single key-value pair"))
		} else {
			result, erl := sp.expression(*mapOfNode.Content[0])
			mapOf.key = result

			errList.Push(erl)

			result, erl = sp.expression(*mapOfNode.Content[1])
			mapOf.value = result

			errList.Push(erl)
		}
	}

	if _merge {
		if mergeNode.Kind != yaml.SequenceNode {
			errList.Push(sp.schemaError(mergeNode, "a YAML sequence of mergeable expressions"))
		} else {
			mergeList := make([]tMergeableExpression, 0, len(mergeNode.Content))
			for _, subNode := range mergeNode.Content {
				expression, erl := sp.expression(*subNode)
				errList.Push(erl)
				if len(erl) != 0 {
					continue
				}

				mergeable, erl := checkMergeable(sp, node, expression)
				errList.Push(erl)
				if len(erl) != 0 {
					continue
				}

				mergeList = append(mergeList, mergeable)
			}
		}
	}

	return tMapForm{
		propertyMap: propertyMap,
		optionalMap: optionalMap,
		mapOf:       mapOf,
		mergeList:   mergeList,
	}, errList.ConcatError()
}

func seqForm(sp tSchemaParser, node yaml.Node, formMap tFormMap) (tSeqForm, []error) {
	errList := errorlist.List{}

	tupleNode, _tuple := formMap["_tuple"]
	optionalNode, _optional := formMap["_optional"]
	seqOfNode, _seqOf := formMap["_seqOf"]

	tupleList := []tExpression{}
	optionalList := []tExpression{}
	var seqOfExpression tExpression = nil

	// _tuple and _optional

	if _tuple {
		tupleList = make([]tExpression, len(tupleNode.Content))
		for k, subNode := range tupleNode.Content {
			res, erl := sp.expression(*subNode)
			errList.Push(erl)
			tupleList[k] = res
		}
	}

	if _optional {
		optionalList = make([]tExpression, len(optionalNode.Content))
		for k, subNode := range tupleNode.Content {
			res, erl := sp.expression(*subNode)
			errList.Push(erl)
			optionalList[k] = res
		}
	}

	if _seqOf {
		res, erl := sp.expression(seqOfNode)
		errList.Push(erl)
		seqOfExpression = res
	}

	return tSeqForm{
		tuple:         tupleList,
		optionalTuple: optionalList,
		seqOf:         seqOfExpression,
	}, errList.ConcatError()
}

func sizingChecker(sp tSchemaParser, node yaml.Node, formMap tFormMap) (tSizing, []error) {
	errList := errorlist.List{}

	minNode, _min := formMap["_min"]
	maxNode, _max := formMap["_max"]
	nbNode, _nb := formMap["_nb"]

	var sizing tSizing = nil

	tryDecodeInteger := func(theNode yaml.Node) int {
		var theInt int
		err := node.Decode(&theInt)
		errList.Push(sp.schemaError(theNode, "an integer, but error happened: "+err.Error()))
		return theInt
	}

	switch {
	case _nb && (_min || _max):
		errList.Push(sp.schemaError(node, "no use of _nb, _min and _max together"))
	case !_min && !_max && !_nb:
		sizing = tSizingNone{}
	case _nb:
		sizing = tSizingNb{nb: tryDecodeInteger(nbNode)}
	case _min && !_max:
		sizing = tSizingMin{min: tryDecodeInteger(minNode)}
	case _max && !_min:
		sizing = tSizingMax{max: tryDecodeInteger(maxNode)}
	case _min && _max:
		sizing = tSizingMinMax{
			tSizingMin{min: tryDecodeInteger(minNode)},
			tSizingMax{max: tryDecodeInteger(maxNode)},
		}
	default:
		errList.Push(sp.schemaError(node, "no unexpected combination of _nb, _min and _max (Internal error? "+pleaseReport+")"))
	}

	return sizing, errList.ConcatError()
}

func oneOfChecker(sp tSchemaParser, _ yaml.Node, formMap tFormMap) (tExpression, []error) {
	oneOfValueNode := formMap["_oneOf"]

	if oneOfValueNode.Kind != yaml.SequenceNode {
		return nil, sp.schemaError(oneOfValueNode, "a sequence (of lidy expressions)")
	}
	errList := errorlist.List{}
	optionList := []tExpression{}

	for _, subNode := range oneOfValueNode.Content {
		expression, erl := sp.expression(*subNode)
		errList.Push(erl)
		optionList = append(optionList, expression)
	}

	return tOneOf{
		optionList: optionList,
	}, errList.ConcatError()
}

func inChecker(sp tSchemaParser, _ yaml.Node, formMap tFormMap) (tExpression, []error) {
	inValueNode := formMap["_in"]

	if inValueNode.Kind != yaml.SequenceNode {
		return nil, sp.schemaError(inValueNode, "a sequence (of YAML scalars)")
	}
	errList := errorlist.List{}
	valueMap := make(map[string][]string)

NodeContentLoop:
	for _, value := range inValueNode.Content {
		// scalar values only
		if value.Kind != yaml.ScalarNode {
			errList.Push(sp.schemaError(inValueNode, "a scalar value"))
			continue
		}

		// add a slice to the map if this kind of scalar value is yet unmet
		if _, ok := valueMap[value.Tag]; !ok {
			valueMap[value.Tag] = []string{}
		}

		for _, v := range valueMap[value.Tag] {
			if v == value.Value {
				errList.Push(sp.schemaError(*value, "no duplicated value"))
				continue NodeContentLoop
			}
		}

		// add the value
		valueMap[value.Tag] = append(valueMap[value.Tag], value.Value)
	}

	return tIn{
		valueMap: valueMap,
	}, errList.ConcatError()
}

func regexChecker(sp tSchemaParser, _ yaml.Node, formMap tFormMap) (tExpression, []error) {
	regexValueNode := formMap["_regex"]
	if regexValueNode.Tag != "!!str" {
		return nil, sp.schemaError(regexValueNode, "a string (a regex)")
	}

	regexString := regexValueNode.Value

	regex, err := regexp.Compile(regexString)

	if err != nil {
		return nil, sp.schemaError(regexValueNode, fmt.Sprintf(
			"a valid regex (error: '%s')",
			err.Error(),
		))
	}

	return tRegex{
		regex: regex,
	}, nil
}