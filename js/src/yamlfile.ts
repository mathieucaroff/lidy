import * as yaml from "yaml"
import { JoinError } from "./error"
import { File } from "./file"

export interface YamlFile {
  file: File
  yaml: yaml.YAMLMap<any, any>
  lineCounter: yaml.LineCounter
  parsingError?: Error
  doneParsing: boolean
}

export function createYamlFile(file: File): YamlFile {
  return {
    file,
    yaml: new yaml.YAMLMap(),
    lineCounter: new yaml.LineCounter(),
    doneParsing: false,
  }
}

export function unmarshalYamlFile(yamlFile: YamlFile): Error | undefined {
  if (yamlFile.doneParsing) {
    return yamlFile.parsingError
  }
  try {
    const document = yaml.parseDocument(yamlFile.file.content, {
      lineCounter: yamlFile.lineCounter,
    })

    yamlFile.yaml = document.contents as yaml.YAMLMap.Parsed
    if (document.errors.length > 0) {
      yamlFile.parsingError = new JoinError(...document.errors)
    }
  } catch (e) {
    yamlFile.parsingError = e
  }

  yamlFile.doneParsing = true
  return yamlFile.parsingError
}
