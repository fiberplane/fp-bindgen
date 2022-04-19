import {
  assertEquals,
  fail,
} from "https://deno.land/std@0.135.0/testing/asserts.ts";
import { loadPlugin } from "./loader.ts";
import type { Imports } from "../example-protocol/bindings/ts-runtime/index.ts";
import type {
  FpAdjacentlyTagged,
  FpFlatten,
  FpInternallyTagged,
  FpPropertyRenaming,
  FpUntagged,
  FpVariantRenaming,
  HttpResult,
  Request,
  SerdeAdjacentlyTagged,
  SerdeFlatten,
  SerdeInternallyTagged,
  SerdePropertyRenaming,
  SerdeUntagged,
  SerdeVariantRenaming,
  StructWithGenerics,
} from "../example-protocol/bindings/ts-runtime/types.ts";

let voidFunctionCalled = false;

const imports: Imports = {
  importFpAdjacentlyTagged: (arg: FpAdjacentlyTagged): FpAdjacentlyTagged => {
    assertEquals(arg, { type: "Bar", payload: "Hello, plugin!" });
    return { type: "Baz", payload: { a: -8, b: 64n } };
  },

  importFpEnum: (arg: FpVariantRenaming): FpVariantRenaming => {
    assertEquals(arg, "foo_bar");
    return {
      QUX_BAZ: {
        FOO_BAR: "foo_bar",
        qux_baz: 64.0,
      },
    };
  },

  importFpFlatten: (arg: FpFlatten): FpFlatten => {
    assertEquals(arg, { foo: "Hello, 🇳🇱!", bar: -64n });
    return { foo: "Hello, 🇩🇪!", bar: -64n };
  },

  importFpInternallyTagged: (arg: FpInternallyTagged): FpInternallyTagged => {
    assertEquals(arg, { type: "Foo" });
    return { type: "Baz", a: -8, b: 64n };
  },

  importFpStruct: (arg: FpPropertyRenaming): FpPropertyRenaming => {
    assertEquals(arg, { fooBar: "foo_bar", QUX_BAZ: 64.0, rawStruct: -32 });
    return { fooBar: "fooBar", QUX_BAZ: -64.0, rawStruct: 32 };
  },

  importFpUntagged: (arg: FpUntagged): FpUntagged => {
    assertEquals(arg, "Hello, plugin!");
    return { a: -8, b: 64n };
  },

  importGenerics: (
    arg: StructWithGenerics<bigint>,
  ): StructWithGenerics<bigint> => {
    assertEquals(
      arg,
      {
        list: [0, 64],
        points: [{ value: 64 }],
        recursive: [{ value: { value: 64 } }],
        complex_nested: {
          "one": [{ value: 1.0 }],
          "two": [{ value: 2.0 }],
        },
        optional_timestamp: "1970-01-01T00:00:00Z",
      },
    );
    return {
      list: [0n, 64n],
      points: [{ value: 64n }],
      recursive: [{ value: { value: 64n } }],
      complex_nested: {
        "een": [{ value: 1.0 }],
        "twee": [{ value: 2.0 }],
      },
      optional_timestamp: "1970-01-01T00:00:00Z",
    };
  },

  importMultiplePrimitives: (arg1: number, arg2: string): bigint => {
    assertEquals(arg1, -8);
    assertEquals(arg2, "Hello, 🇳🇱!");
    return -64n;
  },

  importPrimitiveI16: (arg: number): number => {
    assertEquals(arg, -16);
    return -16;
  },

  importPrimitiveI32: (arg: number): number => {
    assertEquals(arg, -32);
    return -32;
  },

  importPrimitiveI64: (arg: bigint): bigint => {
    assertEquals(arg, -64n);
    return -64n;
  },

  importPrimitiveI8: (arg: number): number => {
    assertEquals(arg, -8);
    return -8;
  },

  importPrimitiveU16: (arg: number): number => {
    assertEquals(arg, 16);
    return 16;
  },

  importPrimitiveU32: (arg: number): number => {
    assertEquals(arg, 32);
    return 32;
  },

  importPrimitiveU64: (arg: bigint): bigint => {
    assertEquals(arg, 64n);
    return 64n;
  },

  importPrimitiveU8: (arg: number): number => {
    assertEquals(arg, 8);
    return 8;
  },

  importSerdeAdjacentlyTagged: (
    arg: SerdeAdjacentlyTagged,
  ): SerdeAdjacentlyTagged => {
    assertEquals(arg, { type: "Bar", payload: "Hello, plugin!" });
    return { type: "Baz", payload: { a: -8, b: 64n } };
  },

  importSerdeEnum: (arg: SerdeVariantRenaming): SerdeVariantRenaming => {
    assertEquals(arg, "foo_bar");
    return {
      QUX_BAZ: {
        FooBar: "foo_bar",
        qux_baz: 64.0,
      },
    };
  },

  importSerdeFlatten: (arg: SerdeFlatten): SerdeFlatten => {
    assertEquals(arg, { foo: "Hello, 🇳🇱!", bar: -64n });
    return { foo: "Hello, 🇩🇪!", bar: -64n };
  },

  importSerdeInternallyTagged: (
    arg: SerdeInternallyTagged,
  ): SerdeInternallyTagged => {
    assertEquals(arg, { type: "Foo" });
    return { type: "Baz", a: -8, b: 64n };
  },

  importSerdeStruct: (arg: SerdePropertyRenaming): SerdePropertyRenaming => {
    assertEquals(arg, { fooBar: "foo_bar", QUX_BAZ: 64.0, rawStruct: -32 });
    return { fooBar: "fooBar", QUX_BAZ: -64.0, rawStruct: 32 };
  },

  importSerdeUntagged: (arg: SerdeUntagged): SerdeUntagged => {
    assertEquals(arg, "Hello, plugin!");
    return { a: -8, b: 64n };
  },

  importString: (arg: string): string => {
    assertEquals(arg, "Hello, plugin!");
    return "Hello, world!";
  },

  importTimestamp: (arg: string): string => {
    assertEquals(arg, "2022-04-12T19:10:00Z");
    return "2022-04-13T12:37:00Z";
  },

  importVoidFunction: (): void => {
    voidFunctionCalled = true;
  },

  log: (message: string): void => {
    // The plugin is not expected to log anything unless it panics:
    fail("Plugin panic: " + message);
  },

  makeHttpRequest: (request: Request): Promise<HttpResult> => {
    const encoder = new TextEncoder();

    assertEquals(request, {
      url: "https://fiberplane.dev",
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: encoder.encode(
        JSON.stringify({ "country": "🇳🇱", "type": "sign-up" }),
      ),
    });
    return Promise.resolve({
      Ok: {
        body: encoder.encode(
          JSON.stringify({ "status": "confirmed" }),
        ),
        headers: {
          "Content-Type": "application/json",
        },
        status_code: 200,
      },
    });
  },
};

Deno.test("test primitives", async () => {
  const plugin = await loadPlugin(
    "../example-plugin/target/wasm32-unknown-unknown/debug/example_plugin.wasm",
    imports,
  );

  assertEquals(plugin.exportPrimitiveU8?.(8), 8);
  assertEquals(plugin.exportPrimitiveU16?.(16), 16);
  assertEquals(plugin.exportPrimitiveU32?.(32), 32);
  assertEquals(plugin.exportPrimitiveU64?.(64n), 64n);
  assertEquals(plugin.exportPrimitiveI8?.(-8), -8);
  assertEquals(plugin.exportPrimitiveI16?.(-16), -16);
  assertEquals(plugin.exportPrimitiveI32?.(-32), -32);
  assertEquals(plugin.exportPrimitiveI64?.(-64n), -64n);
});

Deno.test("test flattened structs", async () => {
  const plugin = await loadPlugin(
    "../example-plugin/target/wasm32-unknown-unknown/debug/example_plugin.wasm",
    imports,
  );

  assertEquals(plugin.exportFpFlatten?.({ foo: "Hello, 🇳🇱!", bar: -64 }), {
    foo: "Hello, 🇩🇪!",
    bar: -64,
  });

  assertEquals(plugin.exportSerdeFlatten?.({ foo: "Hello, 🇳🇱!", bar: -64 }), {
    foo: "Hello, 🇩🇪!",
    bar: -64,
  });
});
