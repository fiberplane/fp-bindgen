import {
  assert,
  assertAlmostEquals,
  assertEquals,
  fail,
} from "https://deno.land/std@0.135.0/testing/asserts.ts";
import { loadPlugin } from "./loader.ts";
import type { Exports, Imports } from "../example-protocol/bindings/ts-runtime/index.ts";
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
    return { type: "Baz", payload: { a: -8, b: 64 } };
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
    return { foo: "Hello, 🇩🇪!", bar: -64 };
  },

  importFpInternallyTagged: (arg: FpInternallyTagged): FpInternallyTagged => {
    assertEquals(arg, { type: "Foo" });
    return { type: "Baz", a: -8, b: 64 };
  },

  importFpStruct: (arg: FpPropertyRenaming): FpPropertyRenaming => {
    assertEquals(arg, { fooBar: "foo_bar", QUX_BAZ: 64.0, rawStruct: -32 });
    return { fooBar: "fooBar", QUX_BAZ: -64.0, rawStruct: 32 };
  },

  importFpUntagged: (arg: FpUntagged): FpUntagged => {
    assertEquals(arg, "Hello, plugin!");
    return { a: -8, b: 64 };
  },

  importGenerics: (
    arg: StructWithGenerics<number>,
  ): StructWithGenerics<number> => {
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
      list: [0, 64],
      points: [{ value: 64 }],
      recursive: [{ value: { value: 64 } }],
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

  importPrimitiveBool: (arg: boolean): boolean => {
    return arg;
  },

  importPrimitiveF32: (arg: number): number => {
    assertAlmostEquals(arg, 3.1415926535);
    return 3.1415926535;
  },

  importPrimitiveF64: (arg: number): number => {
    assertAlmostEquals(arg, 2.718281828459);
    return 2.718281828459;
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
    return { type: "Baz", payload: { a: -8, b: 64 } };
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
    return { foo: "Hello, 🇩🇪!", bar: -64 };
  },

  importSerdeInternallyTagged: (
    arg: SerdeInternallyTagged,
  ): SerdeInternallyTagged => {
    assertEquals(arg, { type: "Foo" });
    return { type: "Baz", a: -8, b: 64 };
  },

  importSerdeStruct: (arg: SerdePropertyRenaming): SerdePropertyRenaming => {
    assertEquals(arg, { fooBar: "foo_bar", QUX_BAZ: 64.0, rawStruct: -32 });
    return { fooBar: "fooBar", QUX_BAZ: -64.0, rawStruct: 32 };
  },

  importSerdeUntagged: (arg: SerdeUntagged): SerdeUntagged => {
    assertEquals(arg, "Hello, plugin!");
    return { a: -8, b: 64 };
  },

  importString: (arg: string): string => {
    assertEquals(arg, "Hello, world!");
    return "Hello, plugin!";
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
      url: "https://fiberplane.dev/",
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

let examplePlugin: Exports | null = null;
async function loadExamplePlugin() {
  if (!examplePlugin) {
    examplePlugin = await loadPlugin(
      "../example-plugin/target/wasm32-unknown-unknown/debug/example_plugin.wasm",
      imports,
    );

    const { init } = examplePlugin;
    assert(init);
    init();
  }

  return examplePlugin;
}

Deno.test("primitives", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportPrimitiveBool?.(true), true);
  assertEquals(plugin.exportPrimitiveBool?.(false), false);

  assertEquals(plugin.exportPrimitiveU8?.(8), 8);
  assertEquals(plugin.exportPrimitiveU16?.(16), 16);
  assertEquals(plugin.exportPrimitiveU32?.(32), 32);
  assertEquals(plugin.exportPrimitiveU64?.(64n), 64n);
  assertEquals(plugin.exportPrimitiveI8?.(-8), -8);
  assertEquals(plugin.exportPrimitiveI16?.(-16), -16);
  assertEquals(plugin.exportPrimitiveI32?.(-32), -32);
  assertEquals(plugin.exportPrimitiveI64?.(-64n), -64n);

  assertEquals(plugin.exportMultiplePrimitives?.(-8, "Hello, 🇳🇱!"), -64n);

  assertAlmostEquals(plugin.exportPrimitiveF32?.(3.1415926535) ?? 0, 3.1415926535);
  assertAlmostEquals(plugin.exportPrimitiveF64?.(2.718281828459) ?? 0, 2.718281828459);
});

Deno.test("string", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportString?.("Hello, plugin!"), "Hello, world!");
});

Deno.test("timestamp", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportTimestamp?.("2022-04-12T19:10:00Z"), "2022-04-13T12:37:00Z");
});

Deno.test("flattened structs", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportFpStruct?.({
    fooBar: "foo_bar",
    QUX_BAZ: 64.0,
    rawStruct: -32
  }), {
    fooBar: "fooBar",
    QUX_BAZ: -64.0,
    rawStruct: 32,
  });

  assertEquals(plugin.exportFpEnum?.("foo_bar"), {
    QUX_BAZ: {
      FOO_BAR: "foo_bar",
      qux_baz: 64.0,
    }
  });

  assertEquals(plugin.exportSerdeStruct?.({
    fooBar: "foo_bar",
    QUX_BAZ: 64.0,
    rawStruct: -32
  }), {
    fooBar: "fooBar",
    QUX_BAZ: -64.0,
    rawStruct: 32,
  });

  assertEquals(plugin.exportSerdeEnum?.("foo_bar"), {
    QUX_BAZ: {
      FooBar: "foo_bar",
      qux_baz: 64.0,
    }
  });
});

Deno.test("generics", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportGenerics?.({
    list: [0, 64],
    points: [{ value: 64 }],
    recursive: [{ value: { value: 64 } }],
    complex_nested: {
      "one": [{ value: 1.0 }],
      "two": [{ value: 2.0 }],
    },
    optional_timestamp: "1970-01-01T00:00:00Z",
  }), {
    list: [0, 64],
    points: [{ value: 64 }],
    recursive: [{ value: { value: 64 } }],
    complex_nested: {
      "een": [{ value: 1.0 }],
      "twee": [{ value: 2.0 }],
    },
    optional_timestamp: "1970-01-01T00:00:00Z",
  });
});

Deno.test("property renaming", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportFpFlatten?.({ foo: "Hello, 🇳🇱!", bar: -64 }), {
    foo: "Hello, 🇩🇪!",
    bar: -64,
  });

  assertEquals(plugin.exportSerdeFlatten?.({ foo: "Hello, 🇳🇱!", bar: -64 }), {
    foo: "Hello, 🇩🇪!",
    bar: -64,
  });
});

Deno.test("tagged enums", async () => {
  const plugin = await loadExamplePlugin();

  assertEquals(plugin.exportFpAdjacentlyTagged?.({ type: "Bar", payload: "Hello, plugin!" }), {
    type: "Baz",
    payload: { a: -8, b: 64 }
  });

  assertEquals(plugin.exportFpInternallyTagged?.({ type: "Foo" }), {
    type: "Baz",
    a: -8,
    b: 64
  });

  assertEquals(plugin.exportFpUntagged?.("Hello, plugin!"), { a: -8, b: 64 });

  assertEquals(plugin.exportSerdeAdjacentlyTagged?.({ type: "Bar", payload: "Hello, plugin!" }), {
    type: "Baz",
    payload: { a: -8, b: 64 }
  });

  assertEquals(plugin.exportSerdeInternallyTagged?.({ type: "Foo" }), {
    type: "Baz",
    a: -8,
    b: 64
  });

  assertEquals(plugin.exportSerdeUntagged?.("Hello, plugin!"), { a: -8, b: 64 });
});

Deno.test("async struct", async () => {
  const { exportAsyncStruct } = await loadExamplePlugin();
  assert(exportAsyncStruct);

  assertEquals(await exportAsyncStruct({
    fooBar: "foo_bar",
    QUX_BAZ: 64.0,
    rawStruct: -32
  }, 64n), {
    fooBar: "fooBar",
    QUX_BAZ: -64.0,
    rawStruct: 32,
  });
});

Deno.test("fetch async data", async () => {
  const { fetchData } = await loadExamplePlugin();
  assert(fetchData);

  const data = await fetchData("sign-up");
  assertEquals(data, {
    Ok: JSON.stringify({ "status": "confirmed" })
  });
});
