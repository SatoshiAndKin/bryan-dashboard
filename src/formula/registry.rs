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
    FuncInfo {
        name: "BLOCK_NUMBER",
        syntax: "BLOCK_NUMBER([chain_id])",
        description: "Returns the latest Ethereum block number. Optional chain_id to verify chain.",
    },
    FuncInfo {
        name: "BLOCK",
        syntax: "BLOCK([chain_id])",
        description: "Alias for BLOCK_NUMBER.",
    },
    FuncInfo {
        name: "BLOCK_HASH",
        syntax: "BLOCK_HASH([chain_id])",
        description: "Returns the latest Ethereum block hash. Optional chain_id to verify chain.",
    },
    FuncInfo {
        name: "BLOCK_TIMESTAMP",
        syntax: "BLOCK_TIMESTAMP([chain_id])",
        description:
            "Returns the latest block timestamp (unix). Optional chain_id to verify chain.",
    },
    FuncInfo {
        name: "BLOCK_BASE_FEE",
        syntax: "BLOCK_BASE_FEE([chain_id])",
        description: "Returns the latest block base fee in wei. Optional chain_id to verify chain.",
    },
    FuncInfo {
        name: "BASE_FEE",
        syntax: "BASE_FEE([chain_id])",
        description: "Alias for BLOCK_BASE_FEE.",
    },
    FuncInfo {
        name: "ETH_CALL",
        syntax: "ETH_CALL(to, data, [chain_id])",
        description: "Execute an eth_call against the connected RPC.",
    },
    FuncInfo {
        name: "BLOCK_AGE",
        syntax: "BLOCK_AGE([chain_id])",
        description: "Returns seconds since the latest block timestamp. Includes milliseconds.",
    },
    FuncInfo {
        name: "ETH_BALANCE",
        syntax: "ETH_BALANCE(address)",
        description: "Returns the ETH balance of an address (in ETH).",
    },
    FuncInfo {
        name: "IF",
        syntax: "IF(condition, then, [else])",
        description: "Returns then if condition is true (non-zero), else otherwise.",
    },
    FuncInfo {
        name: "MIN",
        syntax: "MIN(value1, [value2], ...)",
        description: "Returns the smallest number, ignoring empty cells.",
    },
    FuncInfo {
        name: "MAX",
        syntax: "MAX(value1, [value2], ...)",
        description: "Returns the largest number, ignoring empty cells.",
    },
    FuncInfo {
        name: "COUNT",
        syntax: "COUNT(value1, [value2], ...)",
        description: "Counts cells containing numbers.",
    },
    FuncInfo {
        name: "COUNTA",
        syntax: "COUNTA(value1, [value2], ...)",
        description: "Counts non-empty cells.",
    },
    FuncInfo {
        name: "ROUND",
        syntax: "ROUND(number, [decimals])",
        description: "Rounds a number to a given number of decimal places.",
    },
    FuncInfo {
        name: "ABS",
        syntax: "ABS(number)",
        description: "Returns the absolute value of a number.",
    },
    FuncInfo {
        name: "FLOOR",
        syntax: "FLOOR(number, [significance])",
        description: "Rounds down to the nearest integer or multiple.",
    },
    FuncInfo {
        name: "CEIL",
        syntax: "CEIL(number, [significance])",
        description: "Rounds up to the nearest integer or multiple.",
    },
    FuncInfo {
        name: "MOD",
        syntax: "MOD(number, divisor)",
        description: "Returns the remainder after division.",
    },
    FuncInfo {
        name: "POWER",
        syntax: "POWER(base, exponent)",
        description: "Returns base raised to a power.",
    },
    FuncInfo {
        name: "SQRT",
        syntax: "SQRT(number)",
        description: "Returns the square root.",
    },
    FuncInfo {
        name: "LN",
        syntax: "LN(number)",
        description: "Returns the natural logarithm.",
    },
    FuncInfo {
        name: "LOG",
        syntax: "LOG(number)",
        description: "Returns the base-10 logarithm.",
    },
    FuncInfo {
        name: "CONCATENATE",
        syntax: "CONCATENATE(text1, text2, ...)",
        description: "Joins text strings. Also: CONCAT() or the & operator.",
    },
    FuncInfo {
        name: "LEFT",
        syntax: "LEFT(text, [count])",
        description: "Returns the first N characters.",
    },
    FuncInfo {
        name: "RIGHT",
        syntax: "RIGHT(text, [count])",
        description: "Returns the last N characters.",
    },
    FuncInfo {
        name: "MID",
        syntax: "MID(text, start, count)",
        description: "Returns characters from the middle of a string (1-based start).",
    },
    FuncInfo {
        name: "LEN",
        syntax: "LEN(text)",
        description: "Returns the length of a string.",
    },
    FuncInfo {
        name: "UPPER",
        syntax: "UPPER(text)",
        description: "Converts text to uppercase.",
    },
    FuncInfo {
        name: "LOWER",
        syntax: "LOWER(text)",
        description: "Converts text to lowercase.",
    },
    FuncInfo {
        name: "TRIM",
        syntax: "TRIM(text)",
        description: "Removes leading and trailing whitespace.",
    },
    FuncInfo {
        name: "TEXT",
        syntax: "TEXT(value)",
        description: "Converts a value to text.",
    },
    FuncInfo {
        name: "VALUE",
        syntax: "VALUE(text)",
        description: "Converts text to a number.",
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
