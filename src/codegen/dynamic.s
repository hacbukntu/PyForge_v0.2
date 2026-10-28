# Valores de lista[mixto]. Cada caja estable contiene etiqueta (0) y contenido (8).
# Etiquetas públicas del runtime: 1 entero, 2 flotante, 3 booleano, 4 cadena,
# 5 nulo, 6 lista, 7 diccionario, 8 tupla, 9 conjunto y 10 objeto.
.section .text

# Reserva una caja estable de 16 bytes dentro de páginas de 4096 bytes. Las
# cajas viven hasta terminar el proceso, igual que otros valores temporales del
# runtime actual. El editor y los ejecutables ST son monohilo, de modo que este
# cursor privado no necesita sincronización todavía.
pf_dynamic_box_alloc:
    mov rax, [rip+.L_dynamic_box_next]
    lea rdx, [rax+16]
    cmp rdx, [rip+.L_dynamic_box_end]
    jbe .L_dynamic_box_ready
    sub rsp, 8
    mov esi, 4096
    call pf_col_alloc
    add rsp, 8
    lea rdx, [rax+4096]
    mov [rip+.L_dynamic_box_end], rdx
    lea rdx, [rax+16]
.L_dynamic_box_ready:
    mov [rip+.L_dynamic_box_next], rdx
    ret

# Devuelve el nombre real de la etiqueta como descriptor UTF-8 inmutable.
pf_dynamic_type:
    mov rax, [rdi]
    cmp rax, 1
    je .L_dynamic_type_int
    cmp rax, 2
    je .L_dynamic_type_float
    cmp rax, 3
    je .L_dynamic_type_bool
    cmp rax, 4
    je .L_dynamic_type_text
    cmp rax, 5
    je .L_dynamic_type_null
    cmp rax, 6
    je .L_dynamic_type_list
    cmp rax, 7
    je .L_dynamic_type_dict
    cmp rax, 8
    je .L_dynamic_type_tuple
    cmp rax, 9
    je .L_dynamic_type_set
    cmp rax, 10
    je .L_dynamic_type_object
    jmp pf_dynamic_type_error
.L_dynamic_type_int:
    lea rax, [rip+.L_dynamic_name_int]
    ret
.L_dynamic_type_float:
    lea rax, [rip+.L_dynamic_name_float]
    ret
.L_dynamic_type_bool:
    lea rax, [rip+.L_dynamic_name_bool]
    ret
.L_dynamic_type_text:
    lea rax, [rip+.L_dynamic_name_text]
    ret
.L_dynamic_type_null:
    lea rax, [rip+.L_dynamic_name_null]
    ret
.L_dynamic_type_list:
    lea rax, [rip+.L_dynamic_name_list]
    ret
.L_dynamic_type_dict:
    lea rax, [rip+.L_dynamic_name_dict]
    ret
.L_dynamic_type_tuple:
    lea rax, [rip+.L_dynamic_name_tuple]
    ret
.L_dynamic_type_set:
    lea rax, [rip+.L_dynamic_name_set]
    ret
.L_dynamic_type_object:
    lea rax, [rip+.L_dynamic_name_object]
    ret

# Convierte el contenido dinámico a texto. Los escalares reutilizan exactamente
# el formato normal; las colecciones y objetos producen su nombre legible.
pf_dynamic_text:
    mov rax, [rdi]
    mov rdi, [rdi+8]
    cmp rax, 1
    je pf_int_text
    cmp rax, 2
    je pf_float_text
    cmp rax, 3
    je pf_bool_text
    cmp rax, 4
    je .L_dynamic_text_identity
    cmp rax, 5
    je .L_dynamic_text_null
    cmp rax, 6
    je .L_dynamic_text_list
    cmp rax, 7
    je .L_dynamic_text_dict
    cmp rax, 8
    je .L_dynamic_text_tuple
    cmp rax, 9
    je .L_dynamic_text_set
    cmp rax, 10
    je .L_dynamic_text_object
    jmp pf_dynamic_type_error
.L_dynamic_text_identity:
    mov rax, rdi
    ret
.L_dynamic_text_null:
    lea rax, [rip+.L_dynamic_name_null]
    ret
.L_dynamic_text_list:
    lea rax, [rip+.L_dynamic_name_list]
    ret
.L_dynamic_text_dict:
    lea rax, [rip+.L_dynamic_name_dict]
    ret
.L_dynamic_text_tuple:
    lea rax, [rip+.L_dynamic_name_tuple]
    ret
.L_dynamic_text_set:
    lea rax, [rip+.L_dynamic_name_set]
    ret
.L_dynamic_text_object:
    lea rax, [rip+.L_dynamic_name_object]
    ret

# Imprime una posición mixta con el mismo formato de mostrar para escalares.
pf_dynamic_print:
    mov rax, [rdi]
    mov rdi, [rdi+8]
    cmp rax, 1
    je pf_print_i64
    cmp rax, 2
    je pf_float_print
    cmp rax, 3
    je pf_print_bool
    cmp rax, 4
    je pf_text_print
    cmp rax, 5
    je .L_dynamic_print_null
    cmp rax, 6
    je .L_dynamic_print_list
    cmp rax, 7
    je .L_dynamic_print_dict
    cmp rax, 8
    je .L_dynamic_print_tuple
    cmp rax, 9
    je .L_dynamic_print_set
    cmp rax, 10
    je .L_dynamic_print_object
    jmp pf_dynamic_type_error
.L_dynamic_print_null:
    lea rdi, [rip+.L_dynamic_name_null]
    jmp pf_text_print
.L_dynamic_print_list:
    lea rdi, [rip+.L_dynamic_name_list]
    jmp pf_text_print
.L_dynamic_print_dict:
    lea rdi, [rip+.L_dynamic_name_dict]
    jmp pf_text_print
.L_dynamic_print_tuple:
    lea rdi, [rip+.L_dynamic_name_tuple]
    jmp pf_text_print
.L_dynamic_print_set:
    lea rdi, [rip+.L_dynamic_name_set]
    jmp pf_text_print
.L_dynamic_print_object:
    lea rdi, [rip+.L_dynamic_name_object]
    jmp pf_text_print

# Conversiones comprobadas. Un tipo incompatible lanza una excepción ST y puede
# capturarse con intentar/capturar; nunca reinterpreta una referencia como número.
pf_dynamic_int:
    mov rax, [rdi]
    mov rdi, [rdi+8]
    cmp rax, 1
    je .L_dynamic_value_identity
    cmp rax, 3
    je .L_dynamic_value_identity
    cmp rax, 2
    je pf_convert_float_int
    cmp rax, 4
    je pf_convert_text_int
    lea rdi, [rip+.L_dynamic_int_error]
    jmp pf_throw
pf_dynamic_float:
    mov rax, [rdi]
    mov rdi, [rdi+8]
    cmp rax, 2
    je .L_dynamic_value_identity
    cmp rax, 1
    je pf_convert_int_float
    cmp rax, 3
    je pf_convert_int_float
    cmp rax, 4
    je pf_convert_text_float
    lea rdi, [rip+.L_dynamic_float_error]
    jmp pf_throw
.L_dynamic_value_identity:
    mov rax, rdi
    ret

# Verdad dinámica: cero, cadena/colección vacía y nulo son falsos; objetos son ciertos.
pf_dynamic_bool:
    mov rax, [rdi]
    mov rdx, [rdi+8]
    cmp rax, 2
    je .L_dynamic_bool_float
    cmp rax, 4
    je .L_dynamic_bool_sized
    cmp rax, 5
    je .L_dynamic_bool_false
    cmp rax, 6
    jb .L_dynamic_bool_word
    cmp rax, 9
    jbe .L_dynamic_bool_sized
    cmp rax, 10
    je .L_dynamic_bool_true
    jmp pf_dynamic_type_error
.L_dynamic_bool_float:
    mov rdi, rdx
    jmp pf_convert_float_bool
.L_dynamic_bool_sized:
    cmp QWORD PTR [rdx], 0
    setne al
    movzx eax, al
    ret
.L_dynamic_bool_word:
    test rdx, rdx
    setne al
    movzx eax, al
    ret
.L_dynamic_bool_true:
    mov eax, 1
    ret
.L_dynamic_bool_false:
    xor eax, eax
    ret

pf_dynamic_type_error:
    lea rdi, [rip+.L_dynamic_invalid_error]
    jmp pf_throw

.section .rodata
.balign 8
.L_dynamic_name_int: .quad 6
    .ascii "entero"
.L_dynamic_name_float: .quad 8
    .ascii "flotante"
.L_dynamic_name_bool: .quad 8
    .ascii "booleano"
.L_dynamic_name_text: .quad 6
    .ascii "cadena"
.L_dynamic_name_null: .quad 4
    .ascii "nulo"
.L_dynamic_name_list: .quad 5
    .ascii "lista"
.L_dynamic_name_dict: .quad 11
    .ascii "diccionario"
.L_dynamic_name_tuple: .quad 5
    .ascii "tupla"
.L_dynamic_name_set: .quad 8
    .ascii "conjunto"
.L_dynamic_name_object: .quad 6
    .ascii "objeto"
.L_dynamic_int_error: .quad 45
    .ascii "el valor mixto no se puede convertir a entero"
.L_dynamic_float_error: .quad 47
    .ascii "el valor mixto no se puede convertir a flotante"
.L_dynamic_invalid_error: .quad 33
    .ascii "etiqueta de lista mixta no valida"

# Cursores del almacén de cajas dinámicas. Permanecen fuera de la interfaz ST.
.section .bss
.balign 8
.L_dynamic_box_next: .zero 8
.L_dynamic_box_end: .zero 8
.section .text
