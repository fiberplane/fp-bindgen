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
