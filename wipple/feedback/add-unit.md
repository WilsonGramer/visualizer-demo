---
call.functionCall
number.functionInFunctionCall(call)
number.type(`Number`)
number.source(numberSource)
input.inputInFunctionCall(call)
input.source(inputSource)
call.span(span)
---

[`inputSource`] is not a number unit, so [`numberSource`] cannot be placed here.

-   Try switching the order of this call so the number comes after the function.
-   When a number is written before a function, the function must have the `[unit]` attribute.
