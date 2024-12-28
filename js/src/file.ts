import fs from "fs"
import { join } from "path"

export interface File {
  name: string
  content: string
}

export function readLocalFile(path: string): File {
  return {
    name: path,
    content: fs.readFileSync(join(__dirname, path), "utf8"),
  }
}
