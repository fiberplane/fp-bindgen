export type ComplexGuestToHost = {
    simple: Simple;
    map: Record<string, Simple>;
};

export type ComplexHostToGuest = {
    simple: Simple;
    list: Array<number>;
};

export type Simple = {
    foo: number;
    bar: string;
};

export async function my_async_imported_function(): Promise<ComplexHostToGuest> {
    // TODO: Impl body
}

export function my_complex_imported_function(a: ComplexGuestToHost): ComplexHostToGuest {
    // TODO: Impl body
}

export function my_plain_imported_function(a: number, b: number): number {
    // TODO: Impl body
}

async function my_async_exported_function(): Promise<ComplexGuestToHost> {
    // TODO: Impl body
}

function my_complex_exported_function(a: ComplexHostToGuest): ComplexGuestToHost {
    // TODO: Impl body
}

function my_plain_exported_function(a: number, b: number): number {
    // TODO: Impl body
}
