const lines = ($, rule) =>
    seq(optional($._line_break), rule, repeat(seq($._line_break, rule)), optional($._line_break));

// Give longer operators a higher precedence (eg. so `->` is preferred over `-`),
// and allow operators to continue onto the next line
const operator = ($, rule, { next_line = true } = {}) =>
    seq(token(prec(rule.length, rule)), ...(next_line ? [optional($._line_break)] : []));

const variadic = ($, separator, rule) =>
    seq(field("element", rule), repeat1(seq(separator, field("element", rule))));

// Line breaks and trailing punctuation are allowed if wrapped in parentheses
const variadic_multiline = ($, separator, rule) =>
    seq(
        "(",
        optional($._line_break),
        choice(
            seq(separator, optional($._line_break)),
            repeat1(seq(field("element", rule), separator, optional($._line_break)))
        ),
        ")"
    );

const binary_operators = [
    ["annotate", 12, "left", { right: ($) => $._type_element }],
    ["as", 11, "left", { right: ($) => $._type_element }],
    ["to", 10, "left"],
    ["by", 9, "left"],
    ["power", 8, "right"],
    ["multiply", 7, "left"],
    ["divide", 7, "left"],
    ["remainder", 7, "left"],
    ["add", 6, "left"],
    ["subtract", 6, "left"],
    ["less_than", 5, "left"],
    ["less_than_or_equal", 5, "left"],
    ["greater_than", 5, "left"],
    ["greater_than_or_equal", 5, "left"],
    ["equal", 5, "left"],
    ["not_equal", 5, "left"],
    ["is", 4, "left", { right: ($) => $._pattern_element }],
    ["and", 3, "left"],
    ["or", 2, "left"],
    ["apply", 1, "left"],
];

module.exports = grammar({
    name: "wipple",

    rules: {
        // Statements

        source_file: ($) => $._statements,

        _statements: ($) => lines($, field("statement", $._statement)),

        _statement: ($) =>
            choice(
                $.type_definition_statement,
                $.trait_definition_statement,
                $.constant_definition_statement,
                $.instance_definition_statement,
                $.assignment_statement,
                $.expression_statement
            ),

        type_definition_statement: ($) =>
            prec(
                1,
                seq(
                    field("comment", repeat($.comment)),
                    field("attribute", repeat($._attribute)),
                    field("name", $.type_name),
                    $.colon_operator,
                    optional($._type_function),
                    "type",
                    optional(seq("{", field("representation", $._type_representation), "}"))
                )
            ),

        _type_representation: ($) =>
            choice($.structure_type_representation, $.enumeration_type_representation),

        structure_type_representation: ($) => lines($, field("field", $.field_definition)),

        field_definition: ($) =>
            seq(field("name", $.variable_name), $.annotate_operator, $._type_annotation),

        enumeration_type_representation: ($) => lines($, field("variant", $.variant_definition)),

        variant_definition: ($) =>
            seq(field("name", $.variant_name), repeat(field("element", $._subtype))),

        trait_definition_statement: ($) =>
            prec(
                1,
                seq(
                    field("comment", repeat($.comment)),
                    field("attribute", repeat($._attribute)),
                    field("name", $.type_name),
                    $.colon_operator,
                    optional($._type_function),
                    "trait",
                    optional(seq(field("type", $._subtype), optional($._where_clause)))
                )
            ),

        constant_definition_statement: ($) =>
            prec(
                1,
                seq(
                    field("comment", repeat($.comment)),
                    field("attribute", repeat($._attribute)),
                    field("name", $.variable_name),
                    $.annotate_operator,
                    $._type_annotation
                )
            ),

        instance_definition_statement: ($) =>
            prec(
                1,
                seq(
                    field("comment", repeat($.comment)),
                    field("attribute", repeat($._attribute)),
                    "instance",
                    "(",
                    field("trait", $.type_name),
                    field("parameter", repeat1($._subtype)),
                    ")",
                    optional($._where_clause),
                    optional(seq($.colon_operator, field("value", $._expression)))
                )
            ),

        assignment_statement: ($) =>
            seq(field("pattern", $._pattern), $.colon_operator, field("value", $._expression)),

        expression_statement: ($) => field("expression", $._expression),

        // Expressions

        _expression: ($) =>
            choice($._expression_element, $.tuple_expression, $.collection_expression),

        _expression_element: ($) =>
            choice(
                $._subexpression,
                $.formatted_text_expression,
                $.call_expression,
                $.do_expression,
                $.when_expression,
                $.intrinsic_expression,
                ...binary_operators.map(([key]) => $[`${key}_expression`]),
                $.function_expression
            ),

        _subexpression: ($) =>
            choice(
                $.placeholder_expression,
                $.variable_name_expression,
                $.type_name_expression,
                $.number_expression,
                $.text_expression,
                $.structure_expression,
                $.block_expression,
                $.unit_expression,
                $._parenthesized_expression
            ),

        _parenthesized_expression: ($) =>
            choice(
                seq("(", $._expression, ")"),
                alias($._multiline_tuple_expression, $.tuple_expression),
                alias($._multiline_collection_expression, $.collection_expression)
            ),

        placeholder_expression: ($) => "_",

        variable_name_expression: ($) => field("variable", $.variable_name),

        type_name_expression: ($) => field("type", $.type_name),

        number_expression: ($) => field("value", $.number),

        text_expression: ($) => field("value", $.text),

        structure_expression: ($) =>
            seq("{", lines($, field("field", $.structure_expression_field)), "}"),

        structure_expression_field: ($) =>
            prec(
                2,
                seq(field("name", $.variable_name), $.colon_operator, field("value", $._expression))
            ),

        block_expression: ($) => prec(1, seq("{", optional($._statements), "}")),

        unit_expression: ($) => seq("(", ")"),

        formatted_text_expression: ($) =>
            prec(1, seq(field("text", $.text), field("input", repeat1($._subexpression)))),

        call_expression: ($) =>
            seq(field("function", $._subexpression), field("input", repeat1($._subexpression))),

        do_expression: ($) => prec(50, seq("do", field("input", $._subexpression))),

        when_expression: ($) =>
            prec(
                50,
                seq("when", field("input", $._subexpression), seq("{", optional($._arms), "}"))
            ),

        _arms: ($) => lines($, field("arm", $.arm)),

        arm: ($) =>
            seq(field("pattern", $._pattern), $.arrow_operator, field("value", $._expression)),

        intrinsic_expression: ($) =>
            seq("intrinsic", field("name", $.text), field("input", repeat($._subexpression))),

        ...Object.fromEntries(
            binary_operators.map(
                ([
                    key,
                    precedence,
                    associativity,
                    {
                        left = ($) => $._expression_element,
                        right = ($) => $._expression_element,
                    } = {},
                ]) => [
                    `${key}_expression`,
                    ($) =>
                        prec[associativity](
                            precedence,
                            seq(
                                field("left", left($)),
                                field("operator", $[`${key}_operator`]),
                                field("right", right($))
                            )
                        ),
                ]
            )
        ),

        tuple_expression: ($) => variadic($, $.semicolon_operator, $._expression_element),

        _multiline_tuple_expression: ($) =>
            variadic_multiline($, $.semicolon_operator, $._expression_element),

        collection_expression: ($) => variadic($, $.comma_operator, $._expression_element),

        _multiline_collection_expression: ($) =>
            variadic_multiline($, $.comma_operator, $._expression_element),

        function_expression: ($) =>
            prec.right(
                0,
                seq(
                    repeat1(field("input", $._subpattern)),
                    $.arrow_operator,
                    field("output", $._expression_element)
                )
            ),

        // Patterns

        _pattern: ($) => choice($._pattern_element, $.tuple_pattern),

        _pattern_element: ($) =>
            choice($._subpattern, $.set_pattern, $.variant_pattern, $.or_pattern),

        _subpattern: ($) =>
            choice(
                $.wildcard_pattern,
                $.variable_pattern,
                $.number_pattern,
                $.text_pattern,
                $.destructure_pattern,
                $.unit_pattern,
                $._parenthesized_pattern
            ),

        _parenthesized_pattern: ($) =>
            choice(
                seq("(", $._pattern, ")"),
                $.annotate_pattern,
                alias($._multiline_tuple_pattern, $.tuple_pattern)
            ),

        wildcard_pattern: ($) => "_",

        variable_pattern: ($) => field("variable", $.variable_name),

        number_pattern: ($) => field("value", $.number),

        text_pattern: ($) => field("value", $.text),

        destructure_pattern: ($) =>
            seq("{", lines($, field("field", $.destructure_pattern_field)), "}"),

        destructure_pattern_field: ($) =>
            seq(field("name", $.variable_name), $.colon_operator, field("value", $._pattern)),

        unit_pattern: ($) => seq("(", ")"),

        tuple_pattern: ($) => variadic($, $.semicolon_operator, $._pattern_element),

        _multiline_tuple_pattern: ($) =>
            variadic_multiline($, $.semicolon_operator, $._pattern_element),

        or_pattern: ($) =>
            prec.left(
                0,
                seq(
                    field("left", $._pattern_element),
                    $.or_operator,
                    field("right", $._pattern_element)
                )
            ),

        annotate_pattern: ($) =>
            seq(
                "(",
                field("left", $._pattern_element),
                $.annotate_operator,
                field("right", $._type_element),
                ")"
            ),

        set_pattern: ($) => seq("set", field("variable", $.variable_name)),

        variant_pattern: ($) =>
            seq(field("variant", $.variant_name), field("element", repeat($._subpattern))),

        // Types

        _type: ($) => choice($._type_element, $.tuple_type),

        _type_element: ($) =>
            choice($._subtype, alias($._parameterized_type, $.named_type), $.function_type),

        _subtype: ($) =>
            choice(
                $.placeholder_type,
                $.parameter_type,
                $.named_type,
                $.block_type,
                $.unit_type,
                $._parenthesized_type
            ),

        _parenthesized_type: ($) =>
            choice(seq("(", $._type, ")"), alias($._multiline_tuple_type, $.tuple_type)),

        placeholder_type: ($) => "_",

        parameter_type: ($) => field("name", $.type_parameter_name),

        named_type: ($) => field("name", $.type_name),

        function_type: ($) =>
            prec.right(
                0,
                seq(
                    field("input", repeat1($._subtype)),
                    $.arrow_operator,
                    field("output", $._type_element)
                )
            ),

        block_type: ($) =>
            seq(
                "{",
                seq(
                    optional($._line_break),
                    field("output", $._type_element),
                    optional($._line_break)
                ),
                "}"
            ),

        unit_type: ($) => seq("(", ")"),

        tuple_type: ($) => variadic($, $.semicolon_operator, $._type_element),

        _multiline_tuple_type: ($) => variadic_multiline($, $.semicolon_operator, $._type_element),

        _parameterized_type: ($) =>
            seq(field("name", $.type_name), field("parameter", repeat1($._subtype))),

        // Type annotations and constraints

        _type_annotation: ($) => seq(field("type", $._type_element), optional($._where_clause)),

        _type_function: ($) =>
            seq(repeat1(field("parameter", $.parameter_type)), $.type_arrow_operator),

        _where_clause: ($) => seq("where", repeat1($._constraint)),

        _constraint: ($) =>
            seq(
                "(",
                field(
                    "constraint",
                    choice($.bound_constraint, $.infer_constraint, $.default_constraint)
                ),
                ")"
            ),

        bound_constraint: ($) =>
            seq(field("trait", $.type_name), field("parameter", repeat1($._subtype))),

        infer_constraint: ($) => seq("infer", field("parameter", $.type_parameter_name)),

        default_constraint: ($) =>
            seq(
                field("parameter", $.type_parameter_name),
                $.colon_operator,
                field("value", $._type_element)
            ),

        // Attributes

        _attribute: ($) => seq("[", choice($.name_attribute, $.assign_attribute), "]"),

        name_attribute: ($) => field("name", $.attribute_name),

        assign_attribute: ($) =>
            seq(
                field("name", $.attribute_name),
                $.colon_operator,
                field("value", $._attribute_value)
            ),

        _attribute_value: ($) => choice($.text, $.number),

        // Tokens

        _line_break: ($) => /\n+/,
        comment: ($) => /--.*\n+/,
        text: ($) => /"[^"]*"/,
        number: ($) => /[+\-]?\d+(?:\.\d+)?/,
        _capital_name: ($) => /(?:\d-)*[A-Z]\w*(?:\-\w+)*[!?]?/,
        _lowercase_name: ($) => /\w+(?:\-\w+)*[!?]?/,

        type_name: ($) => $._capital_name,
        variant_name: ($) => $._capital_name,
        variable_name: ($) => $._lowercase_name,
        type_parameter_name: ($) => $._lowercase_name,
        attribute_name: ($) => $._lowercase_name,

        colon_operator: ($) => operator($, ":"),
        type_arrow_operator: ($) => operator($, "=>"),

        annotate_operator: ($) => operator($, "::"),
        as_operator: ($) => operator($, "as"),
        to_operator: ($) => operator($, "to"),
        by_operator: ($) => operator($, "by"),
        power_operator: ($) => operator($, "^"),
        multiply_operator: ($) => operator($, "*"),
        divide_operator: ($) => operator($, "/"),
        remainder_operator: ($) => operator($, "%"),
        add_operator: ($) => operator($, "+"),
        subtract_operator: ($) => operator($, "-"),
        less_than_operator: ($) => operator($, "<"),
        less_than_or_equal_operator: ($) => operator($, "<="),
        greater_than_operator: ($) => operator($, ">"),
        greater_than_or_equal_operator: ($) => operator($, ">="),
        equal_operator: ($) => operator($, "="),
        not_equal_operator: ($) => operator($, "/="),
        is_operator: ($) => operator($, "is"),
        and_operator: ($) => operator($, "and"),
        or_operator: ($) => operator($, "or"),
        apply_operator: ($) => operator($, "."),
        semicolon_operator: ($) => operator($, ";", { next_line: false }),
        comma_operator: ($) => operator($, ",", { next_line: false }),
        arrow_operator: ($) => operator($, "->"),
    },

    word: ($) => $._lowercase_name,

    reserved: {
        global: ($) => [
            "do",
            "infer",
            "instance",
            "intrinsic",
            "set",
            "trait",
            "type",
            "when",
            "where",
        ],
    },

    extras: ($) => [/[ \t]+/, $.comment],

    conflicts: ($) => [
        [$.type_name, $.variant_name],
        [$.tuple_expression, $._multiline_tuple_expression],
        [$.collection_expression, $._multiline_collection_expression],
        [$._multiline_tuple_expression, $._multiline_tuple_pattern],
        [$.placeholder_expression, $.wildcard_pattern],
        [$.variable_name_expression, $.variable_pattern],
        [$.number_expression, $.number_pattern],
        [$.text_expression, $.text_pattern],
        [$.unit_expression, $.unit_pattern],
        [$.tuple_pattern, $._multiline_tuple_pattern],
        [$.tuple_type, $._multiline_tuple_type],
        [$.named_type, $._parameterized_type],
    ],
});
