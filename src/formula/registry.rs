pub struct FuncInfo {
    pub name: &'static str,
    pub syntax: &'static str,
    pub description: &'static str,
}

pub const BUILTIN_FUNCTIONS: &[FuncInfo] = &[
    FuncInfo {
        name: "SUM",
        syntax: "SUM(value1, [value2], ...)",
        description: "Adds all numbers in a range of cells.",
    },
    FuncInfo {
        name: "AVG",
        syntax: "AVG(value1, [value2], ...)",
        description: "Returns the average of numeric values, ignoring empty cells.",
    },
    FuncInfo {
        name: "AVERAGE",
        syntax: "AVERAGE(value1, [value2], ...)",
        description: "Alias for AVG.",
    },
];

pub const OPERATORS: &[FuncInfo] = &[
    FuncInfo {
        name: "+",
        syntax: "A + B",
        description: "Addition",
    },
    FuncInfo {
        name: "-",
        syntax: "A - B",
        description: "Subtraction",
    },
    FuncInfo {
        name: "*",
        syntax: "A * B",
        description: "Multiplication",
    },
    FuncInfo {
        name: "/",
        syntax: "A / B",
        description: "Division",
    },
];

pub const REFERENCES: &[FuncInfo] = &[
    FuncInfo {
        name: "Cell Ref",
        syntax: "A1",
        description: "Reference a single cell by column letter and row number.",
    },
    FuncInfo {
        name: "Range",
        syntax: "A1:B5",
        description: "Reference a rectangular range of cells.",
    },
];
