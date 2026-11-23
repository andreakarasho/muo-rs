// rhai for mdBook's highlight.js 10 (it has no rhai grammar): registers one, then highlights the ```rhai blocks
// book.js already passed over as plain text.
(function () {
    if (typeof hljs === "undefined") return;
    hljs.registerLanguage("rhai", function (hljs) {
        var SUBST = { className: "subst", begin: /\$\{/, end: /\}/, keywords: "true false" };
        return {
            name: "rhai",
            keywords: {
                keyword:
                    "fn let const if else switch while loop for in do until break continue return throw try catch " +
                    "import export as private global this is_shared",
                literal: "true false",
                built_in: "type_of print debug Fn call curry",
            },
            contains: [
                { className: "meta", begin: /\/\/\/ *@\w+/ }, // `/// @command`: before the line comment, which would win
                hljs.C_LINE_COMMENT_MODE,
                hljs.C_BLOCK_COMMENT_MODE,
                hljs.QUOTE_STRING_MODE,
                { className: "string", begin: "`", end: "`", contains: [hljs.BACKSLASH_ESCAPE, SUBST] },
                { className: "string", begin: /'(\\.|[^'\\])'/ },
                { className: "number", begin: /\b(0x[0-9a-fA-F_]+|0b[01_]+|0o[0-7_]+|\d[\d_]*(\.\d[\d_]*)?([eE][+-]?\d+)?)/ },
                { className: "title", begin: /\b[a-z_][a-zA-Z0-9_]*(?=\s*\()/, relevance: 0 },
                { className: "params", begin: /\|/, end: /\|/, keywords: "", relevance: 0 },
            ],
        };
    });
    document.querySelectorAll("code.language-rhai").forEach(function (block) {
        block.textContent = block.textContent; // drop what the plain-text pass left
        hljs.highlightBlock(block);
    });
})();
