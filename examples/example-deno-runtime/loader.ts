import {
  createRuntime,
  type Imports,
} from "../example-protocol/bindings/ts-runtime/index.ts";

export async function loadPlugin(path: string, imports: Imports) {
  // This uses the Deno API to load a local file, but you might want to use
  // `fetch()` here if you're targeting the browser.
  //
  // Example:
  // ```
  // const response = await fetch(url);
  // const plugin = await response.arrayBuffer();
  // ```

  const plugin = await Deno.readFile(path);
  return createRuntime(plugin, imports);
}
