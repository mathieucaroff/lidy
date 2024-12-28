import * as yaml from "yaml"
import { CheckError, JoinError } from "./error"
import { applyExpression } from "./expression"
import { ParserData } from "./lidy"
import { makeResult, MapData, Result } from "./result"

interface MapInfo {
  mandatoryKeys: Record<string, boolean>
  map: Record<string, any>
}

export function resolveMergeReference(
  parserData: ParserData,
  node: yaml.YAMLMap<any, any> | yaml.Scalar<string>,
): yaml.YAMLMap<any, any> {
  if (yaml.isMap<any, any>(node)) {
    return node
  }
  if (!yaml.isScalar<any>(node)) {
    throw new Error(
      "The merge values must be mappings or references to mappings",
    )
  }
  const rule = parserData.parser.ruleSet[node.value]
  if (!rule) {
    throw new Error("The merge value reference must exist in the schema")
  }
  return resolveMergeReference(parserData, rule.node)
}

export function contributeToMapInfo(
  parserData: ParserData,
  mapInfoRef: MapInfo,
  _map: yaml.YAMLMap<any, any>,
  _mapFacultative: yaml.YAMLMap<any, any>,
  _merge: yaml.YAMLSeq<any>,
) {
  if (_merge) {
    _merge.items.forEach((node) => {
      const resolvedNode = resolveMergeReference(parserData, node)
      const mapNode = resolvedNode.get("_map")
      const mapFacultativeNode = resolvedNode.get("_mapFacultative")
      const mergeNode = resolvedNode.get("_merge")
      contributeToMapInfo(
        parserData,
        mapInfoRef,
        mapNode,
        mapFacultativeNode,
        mergeNode,
      )
    })
  }
  if (_map) {
    _map.items.forEach(({ key, value }) => {
      mapInfoRef.map[key] = value
      mapInfoRef.mandatoryKeys[key] = true
    })
  }
  if (_mapFacultative) {
    _mapFacultative.items.forEach(({ key, value }) => {
      if (!mapInfoRef.mandatoryKeys[key]) {
        // We only update the map if the key is not mandatory:
        // A facultative key cannot override a mandatory one.
        mapInfoRef.map[key] = value
      }
    })
  }
}

export function applyMapMatcher(
  parserData: ParserData,
  _map: yaml.YAMLMap<any, any>,
  _mapFacultative: yaml.YAMLMap<any, any>,
  _mapOf: yaml.YAMLMap<any, any>,
  _merge: yaml.YAMLSeq<any>,
  content: yaml.Node,
): Result<MapData<any, any>> {
  if (!yaml.isMap<any, any>(content)) {
    throw new CheckError("_map*", "must be a mapping node", parserData, content)
  }

  const mapInfo: MapInfo = {
    mandatoryKeys: {},
    map: {},
  }
  contributeToMapInfo(parserData, mapInfo, _map, _mapFacultative, _merge)

  const mapData: MapData<any, any> = {
    map: {},
    mapOf: [],
  }
  const joinErrors = new JoinError()

  const mapContent: Record<string, any> = {}
  content.items.forEach(({ key, value }) => {
    if (yaml.isScalar<string>(key) && typeof key.value === "string") {
      mapContent[key.value] = value
    }
  })

  Object.keys(mapInfo.mandatoryKeys).forEach((key) => {
    if (!mapContent[key]) {
      joinErrors.add(
        new CheckError(
          "_map",
          `missing key '${key}' in mapping`,
          parserData,
          content,
        ),
      )
    }
  })

  content.items.forEach((item) => {
    let unknownKey = true
    if (yaml.isScalar<any>(item.key)) {
      const schema = mapInfo.map[item.key.value]
      if (schema) {
        unknownKey = false
        let result: Result<any> | undefined
        try {
          result = applyExpression(parserData, schema, item.value as yaml.Node)
        } catch (e) {
          joinErrors.add(new Error(`key ${item.key.value}: ${e.message}`))
        }
        if (result) {
          mapData.map[item.key.value] = result
        }
      }
    }
    if (unknownKey) {
      if (_mapOf) {
        const maybeJoinError = new JoinError()
        maybeJoinError.add(
          new CheckError(
            "_mapOf",
            `none of the ${_mapOf.items.length} _mapOf association(s) matched`,
            parserData,
            _mapOf,
          ),
        )
        let matchFound = false

        _mapOf.items.some(({ key: schemaKey, value: schemaValue }) => {
          let keyResult: Result<any> | undefined
          let valueResult: Result<any> | undefined
          try {
            keyResult = applyExpression(
              parserData,
              schemaKey,
              item.key as yaml.Node,
            )
          } catch (e) {
            maybeJoinError.add(
              new CheckError(
                "_mapOf[key]",
                e.message,
                parserData,
                item.key as yaml.Node,
              ),
            )
          }
          try {
            valueResult = applyExpression(
              parserData,
              schemaValue,
              item.value as yaml.Node,
            )
          } catch (e) {
            maybeJoinError.add(
              new CheckError(
                "_mapOf[value]",
                e.message,
                parserData,
                item.value as yaml.Node,
              ),
            )
            maybeJoinError.add(e)
          }
          if (keyResult && valueResult) {
            mapData.mapOf.push({ key: keyResult, value: valueResult })
            matchFound = true
            return true
          }
        })
        if (!matchFound) {
          joinErrors.add(maybeJoinError)
        }
      } else {
        if (!yaml.isScalar<any>(item.key)) {
          joinErrors.add(
            new CheckError(
              "_map*",
              `expected a scalar key in mapping`,
              parserData,
              item.key as yaml.Node,
            ),
          )
        } else {
          joinErrors.add(
            new CheckError(
              "_map*",
              `unknown key '${item.key.value}' in mapping`,
              parserData,
              item.key as yaml.Node,
            ),
          )
        }
      }
    }
  })

  joinErrors.throw()
  return makeResult(parserData, content, mapData)
}
