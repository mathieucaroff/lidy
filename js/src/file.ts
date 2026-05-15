import fs from "fs"
import { isAbsolute, resolve } from "path"

export interface File {
  name: string
  content: string
}

/** @param roots Optional array of root directories to resolve the file path against */
export function readLocalFile(path: string, roots?: string[]): File {
  const candidatePaths = isAbsolute(path)
    ? [path]
    : (roots ?? [__dirname, process.cwd()]).map((root) => resolve(root, path))

  for (const candidatePath of candidatePaths) {
    if (!fs.existsSync(candidatePath)) {
      continue
    }

    return {
      name: candidatePath,
      content: fs.readFileSync(candidatePath, "utf8"),
    }
  }

  throw new Error(
    `could not read local file '${path}', tried: ${candidatePaths.join(", ")}`,
  )
}
