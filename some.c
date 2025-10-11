bagay <=> Bagay
Tao <=> Identifier
{ <=> LeftBrace
edad <=> Identifier
: <=> Colon
i32 <=> Identifier
, <=> Comma
tuig <=> Identifier
: <=> Colon
u32 <=> Identifier
, <=> Comma
} <=> RightBrace
itupad <=> Itupad
Tao <=> Identifier
{ <=> LeftBrace
paraan <=> Paraan
get_tuig <=> Identifier
( <=> LeftParen
ako <=> Identifier
) <=> RightParen
-> <=> ThinArrow
u32 <=> Identifier
{ <=> LeftBrace
ibalik <=> Ibalik
ako <=> Identifier
. <=> Dot
tuig <=> Identifier
; <=> SemiColon
} <=> RightBrace
paraan <=> Paraan
bago <=> Identifier
( <=> LeftParen
edad <=> Identifier
: <=> Colon
i32 <=> Identifier
, <=> Comma
tuig <=> Identifier
: <=> Colon
u32 <=> Identifier
) <=> RightParen
-> <=> ThinArrow
Tao <=> Identifier
{ <=> LeftBrace
ibalik <=> Ibalik
Tao <=> Identifier
{ <=> LeftBrace
edad <=> Identifier
: <=> Colon
edad <=> Identifier
, <=> Comma
tuig <=> Identifier
: <=> Colon
tuig <=> Identifier
, <=> Comma
} <=> RightBrace
; <=> SemiColon
} <=> RightBrace
} <=> RightBrace
bagay <=> Bagay
Hayop <=> Identifier
{ <=> LeftBrace
edad <=> Identifier
: <=> Colon
u8 <=> Identifier
, <=> Comma
} <=> RightBrace
paraan <=> Paraan
una <=> Identifier
( <=> LeftParen
) <=> RightParen
{ <=> LeftBrace
ang <=> Ang
tao <=> Identifier
: <=> Colon
Tao <=> Identifier
= <=> Equal
Tao <=> Identifier
:: <=> ColonColon
bago <=> Identifier
( <=> LeftParen
8 <=> IntLit
, <=> Comma
10 <=> IntLit
) <=> RightParen
; <=> SemiColon
ang <=> Ang
edad <=> Identifier
: <=> Colon
i32 <=> Identifier
= <=> Equal
tao <=> Identifier
. <=> Dot
edad <=> Identifier
; <=> SemiColon
ang <=> Ang
tuig <=> Identifier
: <=> Colon
u32 <=> Identifier
= <=> Equal
tao <=> Identifier
. <=> Dot
get_tuig <=> Identifier
( <=> LeftParen
) <=> RightParen
; <=> SemiColon
@ <=> At
println <=> Identifier
( <=> LeftParen
Kumusta mundo <=> StringLit
) <=> RightParen
; <=> SemiColon
@ <=> At
alis <=> Identifier
( <=> LeftParen
1 <=> IntLit
) <=> RightParen
; <=> SemiColon
} <=> RightBrace
Eof <=> Eof
[{"print": ParSymbol { name: "print", param_types: [Sinulid], return_type: Wala }, "Tao": BagaySymbol { name: "Tao" }, "Hayop": BagaySymbol { name: "Hayop" }, "println": ParSymbol { name: "println", param_types: [Sinulid], return_type: Wala }, "alis": ParSymbol { name: "alis", param_types: [I32], return_type: Wala }}, {"get_tuig": MetSymbol { is_static: false, name: "get_tuig", param_types: [Bagay("Tao")], return_type: U32 }}, {"ako": VarSymbol { name: "ako", tol_type: Bagay("Tao") }}]
[{"print": ParSymbol { name: "print", param_types: [Sinulid], return_type: Wala }, "Tao": BagaySymbol { name: "Tao" }, "Hayop": BagaySymbol { name: "Hayop" }, "println": ParSymbol { name: "println", param_types: [Sinulid], return_type: Wala }, "alis": ParSymbol { name: "alis", param_types: [I32], return_type: Wala }}, {"bago": MetSymbol { is_static: true, name: "bago", param_types: [I32, U32], return_type: UnknownIdentifier("Tao") }, "get_tuig": MetSymbol { is_static: false, name: "get_tuig", param_types: [Bagay("Tao")], return_type: U32 }}, {"tuig": VarSymbol { name: "tuig", tol_type: U32 }, "edad": VarSymbol { name: "edad", tol_type: I32 }}]
Tao
UnknownIdentifier("Tao"), Bagay("Tao")
i32
I32, I32
u32
U32, U32
[Bagay("Hayop"), Bagay("Tao")]
#include<stdio.h>
#include<stdlib.h>
typedef struct Tao{int32_t edad;uint32_t tuig;}Tao;uint32_t get_tuig(Tao ako){return (ako.tuig);}Tao bago(int32_t edad,uint32_t tuig){return (struct Tao){.edad=edad,.tuig=tuig};}typedef struct Hayop{uint8_t edad;}Hayop;void __TOL_main__(){const Tao tao = bago(8, 10);const int32_t edad = (tao.edad);const uint32_t tuig = get_tuig(tao);puts("Kumusta mundo");;}int main(){__TOL_main__();return 0;}
Binary compiled: ./exe
