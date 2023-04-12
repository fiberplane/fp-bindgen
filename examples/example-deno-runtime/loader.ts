import {
  createRuntime,
  type Imports,
} from "../example-protocol/bindings/ts-runtime/index.ts";

export async function loadPlugin(path: string, imports: Imports) {
  // This uses the Deno API to load a plugin from a local file.
  //
  // Note that for this use case we generated the TypeScript runtime without
  // support for streaming instantiation. This way, `createRuntime()` accepts
  // an `ArrayBuffer` such as returned by `Deno.readFile()`.
  //
  // If you're targeting browsers instead, you probably want to leave streaming
  // instantiation enabled, so that you can use it as follows:
  //
  // ```
  // const runtime = await createRuntime(fetch(url), imports);
  // ```

  const plugin = await Deno.readFile(path);
  return createRuntime(plugin, imports);
}
