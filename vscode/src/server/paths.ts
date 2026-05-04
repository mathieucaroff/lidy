import * as path from "path"

export function findWorkspaceFolder(
  filePath: string,
  workspaceFolders: string[],
): string | undefined {
  const sorted = [...workspaceFolders].sort(
    (left, right) => right.length - left.length,
  )
  return sorted.find((folder) => filePath.startsWith(folder))
}

export function uriToFsPath(uri: string): string | undefined {
  if (!uri.startsWith("file:")) {
    return undefined
  }
  return decodeURIComponent(uri.replace("file:///", "")).replace(
    /\//g,
    path.sep,
  )
}

export function fsPathToUri(filePath: string): string {
  return "file:///" + toPosix(filePath)
}

export function toPosix(value: string): string {
  return value.replace(/\\/g, "/")
}
