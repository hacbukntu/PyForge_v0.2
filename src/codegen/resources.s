# Puentes System V AMD64: comprueban el estado una vez que Rust ha retornado.
# El salto a pf_throw no atraviesa marcos Rust. El convenio conserva registros no volátiles.
.section .text
pf_resource_open:
    sub rsp, 8
    call pf_native_open
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_buffer:
    sub rsp, 8
    call pf_native_buffer
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_from_text:
    sub rsp, 8
    call pf_native_from_text
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_read:
    sub rsp, 8
    call pf_native_read
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_write:
    sub rsp, 8
    call pf_native_write
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_len:
    sub rsp, 8
    call pf_native_len
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_get:
    sub rsp, 8
    call pf_native_get
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_set:
    sub rsp, 8
    call pf_native_set
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
pf_resource_text:
    push rdi
    call pf_native_utf8
    pop rdi
    test rax, rax
    js pf_resource_error
    jmp pf_native_text
pf_resource_close:
    sub rsp, 8
    call pf_native_close
    add rsp, 8
    test rax, rax
    js pf_resource_error
    ret
# Un único diagnóstico recuperable; no expone direcciones o contenido de archivos.
pf_resource_error:
    lea rdi, [rip+pf_resource_message]
    jmp pf_throw
.section .rodata
pf_resource_message:
.quad 73
.ascii "recursos: operacion nativa fallida; comprueba ruta, modo, limites y UTF-8"
.section .text
