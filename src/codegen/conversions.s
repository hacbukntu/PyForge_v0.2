# Conversiones ST sin libc. Registros volátiles; una palabra de entrada/salida.
.section .text
# Convierte i64 (o booleano 0/1) a f64, con el redondeo IEEE-754 habitual.
pf_convert_int_float:
    cvtsi2sd xmm0, rdi
    movq rax, xmm0
    ret
# Trunca hacia cero, rechazando NaN, infinito y valores fuera de [-2^63, 2^63).
pf_convert_float_int:
    movq xmm0, rdi
    ucomisd xmm0, xmm0
    jp pf_conversion_error
    comisd xmm0, [rip+.L_conversion_min]
    jb pf_conversion_error
    comisd xmm0, [rip+.L_conversion_max]
    jae pf_conversion_error
    cvttsd2si rax, xmm0
    ret
# Cero, incluido -0.0, es falso; los demás valores son verdaderos (también NaN).
pf_convert_float_bool:
    movq xmm0, rdi
    xorpd xmm1, xmm1
    ucomisd xmm0, xmm1
    setne al
    setp dl
    or al, dl
    movzx eax, al
    ret
# Recorta blancos ASCII externos y consume un signo opcional.
# Retorna rsi=inicio, rdx=fin exclusivo, r8=negativo; rechaza texto vacío.
pf_conversion_trim:
    lea rsi, [rdi+8]
    mov rdx, [rdi]
    add rdx, rsi
.L_conversion_left:
    cmp rsi, rdx
    jae pf_conversion_error
    movzx eax, BYTE PTR [rsi]
    cmp eax, 32
    je .L_conversion_skip_left
    sub eax, 9
    cmp eax, 4
    ja .L_conversion_right
.L_conversion_skip_left:
    inc rsi
    jmp .L_conversion_left
.L_conversion_right:
    movzx eax, BYTE PTR [rdx-1]
    cmp eax, 32
    je .L_conversion_skip_right
    sub eax, 9
    cmp eax, 4
    ja .L_conversion_sign
.L_conversion_skip_right:
    dec rdx
    jmp .L_conversion_right
.L_conversion_sign:
    xor r8d, r8d
    cmp BYTE PTR [rsi], '-'
    jne .L_conversion_plus
    mov r8d, 1
    inc rsi
    jmp .L_conversion_nonempty
.L_conversion_plus:
    cmp BYTE PTR [rsi], '+'
    jne .L_conversion_nonempty
    inc rsi
.L_conversion_nonempty:
    cmp rsi, rdx
    jae pf_conversion_error
    ret
# Acumula en negativo: así el mínimo i64 es representable durante toda la conversión.
pf_convert_text_int:
    sub rsp, 8
    call pf_conversion_trim
    xor r10d, r10d
.L_conversion_integer:
    movzx eax, BYTE PTR [rsi]
    sub eax, '0'
    cmp eax, 9
    ja pf_conversion_error
    imul r10, r10, 10
    jo pf_conversion_error
    sub r10, rax
    jo pf_conversion_error
    inc rsi
    cmp rsi, rdx
    jb .L_conversion_integer
    mov rax, r10
    test r8, r8
    jnz .L_conversion_integer_done
    neg rax
    jo pf_conversion_error
.L_conversion_integer_done:
    add rsp, 8
    ret
# Decimal ASCII con punto/exponente. Conserva hasta 19 dígitos significativos.
# La escala compensa dígitos omitidos; el trabajo de escalado está acotado a 400 pasos.
pf_convert_text_float:
    sub rsp, 8
    call pf_conversion_trim
    xorpd xmm0, xmm0
    xor r9d, r9d                 # Escala decimal.
    xor r10d, r10d               # Dígitos significativos retenidos.
    xor r11d, r11d               # Punto encontrado.
    xor ecx, ecx                # Total de dígitos.
.L_conversion_decimal:
    cmp rsi, rdx
    jae .L_conversion_scale
    movzx eax, BYTE PTR [rsi]
    cmp eax, '.'
    je .L_conversion_dot
    cmp eax, 'e'
    je .L_conversion_exponent
    cmp eax, 'E'
    je .L_conversion_exponent
    sub eax, '0'
    cmp eax, 9
    ja pf_conversion_error
    inc rcx
    cmp r10, 19
    jae .L_conversion_extra_digit
    mulsd xmm0, [rip+.L_conversion_ten]
    cvtsi2sd xmm1, rax
    addsd xmm0, xmm1
    test r10, r10
    jnz .L_conversion_count_digit
    test eax, eax
    jz .L_conversion_fraction
.L_conversion_count_digit:
    inc r10
.L_conversion_fraction:
    sub r9, r11
    jmp .L_conversion_next_digit
.L_conversion_extra_digit:
    test r11, r11
    jnz .L_conversion_next_digit
    inc r9
.L_conversion_next_digit:
    inc rsi
    jmp .L_conversion_decimal
.L_conversion_dot:
    test r11, r11
    jnz pf_conversion_error
    mov r11d, 1
    inc rsi
    jmp .L_conversion_decimal
.L_conversion_exponent:
    test rcx, rcx
    jz pf_conversion_error
    inc rsi
    xor r10d, r10d
    mov r11, 1
    cmp rsi, rdx
    jae pf_conversion_error
    cmp BYTE PTR [rsi], '-'
    jne .L_conversion_exp_plus
    mov r11, -1
    inc rsi
    jmp .L_conversion_exp_digits
.L_conversion_exp_plus:
    cmp BYTE PTR [rsi], '+'
    jne .L_conversion_exp_digits
    inc rsi
.L_conversion_exp_digits:
    cmp rsi, rdx
    jae pf_conversion_error
.L_conversion_exp_loop:
    movzx eax, BYTE PTR [rsi]
    sub eax, '0'
    cmp eax, 9
    ja pf_conversion_error
    imul r10, r10, 10
    add r10, rax
    cmp r10, 10000
    ja pf_conversion_error
    inc rsi
    cmp rsi, rdx
    jb .L_conversion_exp_loop
    imul r10, r11
    add r9, r10
.L_conversion_scale:
    test rcx, rcx
    jz pf_conversion_error
    # Cero no desborda aunque su escala sea grande.
    xorpd xmm1, xmm1
    ucomisd xmm0, xmm1
    je .L_conversion_float_sign
    cmp r9, 400
    jg pf_conversion_error
    cmp r9, -400
    jl .L_conversion_underflow
.L_conversion_scale_loop:
    test r9, r9
    jz .L_conversion_float_sign
    js .L_conversion_scale_down
    mulsd xmm0, [rip+.L_conversion_ten]
    dec r9
    jmp .L_conversion_scale_loop
.L_conversion_scale_down:
    divsd xmm0, [rip+.L_conversion_ten]
    inc r9
    jmp .L_conversion_scale_loop
.L_conversion_underflow:
    xorpd xmm0, xmm0
.L_conversion_float_sign:
    movq rax, xmm0
    mov r10, rax
    shr r10, 52
    cmp r10, 2047
    je pf_conversion_error
    test r8, r8
    jz .L_conversion_float_done
    btc rax, 63
.L_conversion_float_done:
    add rsp, 8
    ret
# El mensaje es un descriptor estático válido; pf_throw restaura la pila del capturador.
pf_conversion_error:
    lea rdi, [rip+.L_conversion_message]
    jmp pf_throw
.section .rodata
.balign 8
.L_conversion_min: .double -9223372036854775808.0
.L_conversion_max: .double 9223372036854775808.0
.L_conversion_ten: .double 10.0
.L_conversion_message:
    .quad .L_conversion_message_end - .L_conversion_message_text
.L_conversion_message_text:
    .ascii "conversion invalida: texto numerico incorrecto o valor fuera de rango"
.L_conversion_message_end:
