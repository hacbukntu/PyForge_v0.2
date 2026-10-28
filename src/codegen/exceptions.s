# Excepciones ST explícitas, sin libc. Cada registro pertenece a un marco vivo.
# Lanzar descarta la pila de llamadas hasta el capturador, restaurando rsp y rbp.
# Los registros volátiles no contienen estado persistente del programa ST.
.section .bss
.align 8
pf_handler: .quad 0
pf_exception: .quad 0
.section .rodata
pf_uncaught: .ascii "Error ST: excepción sin capturar: "
.set pf_uncaught_len, .-pf_uncaught
pf_exception_newline: .byte 10
.section .text
pf_throw:
    mov [rip+pf_exception], rdi
    mov r10, [rip+pf_handler]
    test r10, r10
    jz .pf_uncaught
    mov rax, [r10]
    mov [rip+pf_handler], rax
    mov rsp, [r10+8]
    mov rbp, [r10+16]
    jmp QWORD PTR [r10+24]
.pf_uncaught:
    lea rsi, [rip+pf_uncaught]
    # La marca OFFSET conserva una constante inmediata tanto en GNU as como en el
    # ensamblador integrado que usa el módulo de compatibilidad LLVM.
    mov edx, OFFSET FLAT:pf_uncaught_len
    call pf_exception_write
    mov rsi, [rip+pf_exception]
    mov rdx, [rsi]
    add rsi, 8
    call pf_exception_write
    lea rsi, [rip+pf_exception_newline]
    mov edx, 1
    call pf_exception_write
.pf_exception_exit:
    mov eax, 60
    mov edi, 1
    syscall

# Diagnóstico completo en stderr, incluso con escrituras parciales o EINTR.
# Si stderr falla, termina sin intentar lanzar otra excepción durante el diagnóstico.
pf_exception_write:
    test rdx, rdx
    jz .pf_exception_written
.pf_exception_write_more:
    mov eax, 1
    mov edi, 2
    syscall
    cmp rax, -4
    je .pf_exception_write_more
    test rax, rax
    jle .pf_exception_exit
    add rsi, rax
    sub rdx, rax
    jnz .pf_exception_write_more
.pf_exception_written:
    ret
