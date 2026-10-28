# Colecciones ST sin libc. Cabecera: longitud (0), capacidad (8), datos (16).
# Cada entrada ocupa 16 bytes: clave interna (0), entero (8).
# Las referencias apuntan a cabeceras estables: crecer no invalida los alias.
# La memoria de objetos se conserva hasta salir; los búferes antiguos se liberan.
.section .text

# Reserva rsi bytes con mmap anónimo. No modifica registros conservados por ABI.
pf_col_alloc:
    # Limita memoria viva a 256 MiB, contabilizada por páginas, antes de pedir mmap.
    test rsi, rsi
    jz pf_col_memory_error
    mov rax, rsi
    add rax, 4095
    jc pf_col_memory_error
    and rax, -4096
    add rax, [rip+pf_heap_used]
    jc pf_col_memory_error
    cmp rax, 268435456
    ja pf_col_memory_error
    xor edi, edi
    mov edx, 3
    mov r10d, 34
    mov r8, -1
    xor r9d, r9d
    mov eax, 9
    syscall
    cmp rax, -4095
    jae pf_col_memory_error
    mov rdx, rsi
    add rdx, 4095
    and rdx, -4096
    add [rip+pf_heap_used], rdx
    ret

# Crea una cabecera vacía. mmap entrega memoria inicializada a cero.
pf_col_new:
    mov esi, 24
    jmp pf_col_alloc

# Devuelve el número de entradas de una lista o diccionario.
pf_col_length:
    mov rax, [rdi]
    ret

# Comprueba si el valor `rsi` aparece en una secuencia tipada. La etiqueta no
# participa porque el análisis estático ya comprobó el tipo buscado.
pf_std_contains_list:
    mov rcx, [rdi]
    mov rax, [rdi+16]
.L_std_contains_list_loop:
    test rcx, rcx
    jz .L_std_contains_list_false
    cmp QWORD PTR [rax+8], rsi
    je .L_std_contains_list_true
    add rax, 16
    dec rcx
    jmp .L_std_contains_list_loop
.L_std_contains_list_false:
    xor eax, eax
    ret
.L_std_contains_list_true:
    mov eax, 1
    ret

# Variante de lista[mixto]: compara etiqueta y contenido para que 1, verdadero
# y los bits de un flotante nunca se confundan aunque compartan una palabra.
pf_std_contains_list_tagged:
    mov rcx, [rdi]
    mov rax, [rdi+16]
.L_std_contains_tagged_loop:
    test rcx, rcx
    jz .L_std_contains_list_false
    cmp QWORD PTR [rax], rdx
    jne .L_std_contains_tagged_next
    cmp QWORD PTR [rax+8], rsi
    je .L_std_contains_list_true
.L_std_contains_tagged_next:
    add rax, 16
    dec rcx
    jmp .L_std_contains_tagged_loop

# El valor dinámico es una caja estable {etiqueta, contenido}; copia ambos al
# convenio de la variante etiquetada sin conservar punteros al búfer de la lista.
pf_std_contains_list_dynamic:
    mov rdx, [rsi]
    mov rsi, [rsi+8]
    jmp pf_std_contains_list_tagged

# Comprueba si la clave `rsi` existe en un diccionario por clave, no por valor.
pf_std_contains_dict:
    mov rcx, [rdi]
    mov rax, [rdi+16]
.L_std_contains_dict_loop:
    test rcx, rcx
    jz .L_std_contains_dict_false
    cmp QWORD PTR [rax], rsi
    je .L_std_contains_dict_true
    add rax, 16
    dec rcx
    jmp .L_std_contains_dict_loop
.L_std_contains_dict_false:
    xor eax, eax
    ret
.L_std_contains_dict_true:
    mov eax, 1
    ret

# Adaptador histórico: una llamada sin etiqueta continúa significando entero.
pf_list_append:
    mov edx, 1
    jmp pf_list_append_tagged

# Guarda la etiqueta en la primera palabra y el contenido en la segunda.
# Entrada: rdi=lista, rsi=valor, rdx=etiqueta.
pf_list_append_tagged:
    mov rcx, rsi
    mov rsi, rdx
    mov rdx, rcx
    jmp pf_col_append

# Copia una caja dinámica dentro de otra lista sin anidar cajas.
pf_list_append_dynamic:
    mov rdx, [rsi]
    mov rsi, [rsi+8]
    jmp pf_list_append_tagged

# Añade (rsi,rdx) al objeto rdi, duplicando capacidad si hace falta.
# Guarda los argumentos en la pila antes de mmap, copia y libera el búfer anterior.
pf_col_append:
    push rbp
    mov rbp, rsp
    sub rsp, 64
    mov [rbp-8], rdi
    mov [rbp-16], rsi
    mov [rbp-24], rdx
    mov rax, [rdi]
    cmp rax, [rdi+8]
    jb .L_col_space
    mov rax, [rdi+8]
    test rax, rax
    jnz .L_col_double
    mov eax, 4
    jmp .L_col_allocate
.L_col_double:
    # Limita el crecimiento antes de que capacidad*16 pueda desbordar.
    mov rcx, 0x03ffffffffffffff
    cmp rax, rcx
    ja pf_col_memory_error
    shl rax, 1
.L_col_allocate:
    mov [rbp-32], rax
    mov rsi, rax
    shl rsi, 4
    call pf_col_alloc
    mov [rbp-40], rax
    mov rdi, [rbp-8]
    mov rsi, [rdi+16]
    mov rcx, [rdi]
    shl rcx, 1
    mov rdi, rax
    cld
    rep movsq
    mov rdi, [rbp-8]
    mov rsi, [rdi+8]
    test rsi, rsi
    jz .L_col_install
    mov rdi, [rdi+16]
    shl rsi, 4
    mov [rbp-48], rsi
    mov eax, 11
    syscall
    test rax, rax
    jnz pf_col_memory_error
    mov rdx, [rbp-48]
    add rdx, 4095
    and rdx, -4096
    sub [rip+pf_heap_used], rdx
.L_col_install:
    mov rdi, [rbp-8]
    mov rax, [rbp-32]
    mov [rdi+8], rax
    mov rax, [rbp-40]
    mov [rdi+16], rax
.L_col_space:
    mov rdi, [rbp-8]
    mov rcx, [rdi]
    shl rcx, 4
    add rcx, [rdi+16]
    mov rax, [rbp-16]
    mov [rcx], rax
    mov rax, [rbp-24]
    mov [rcx+8], rax
    inc QWORD PTR [rdi]
    mov rax, rdi
    leave
    ret

# Normaliza un índice negativo respecto a la longitud y comprueba ambos límites.
# Devuelve la dirección de la entrada en rax; no modifica el valor rdx de Set.
pf_list_slot:
    test rsi, rsi
    jns .L_list_positive
    add rsi, [rdi]
.L_list_positive:
    cmp rsi, [rdi]
    jae pf_col_index_error
    mov rax, rsi
    shl rax, 4
    add rax, [rdi+16]
    ret

# Lee el entero en lista[índice].
pf_list_get:
    call pf_list_slot
    mov rax, [rax+8]
    ret

# Devuelve una caja estable de 16 bytes. No entrega la dirección del elemento:
# el crecimiento posterior puede sustituir y liberar el búfer interno.
pf_list_get_dynamic:
    call pf_list_slot
    sub rsp, 24
    mov r10, [rax]
    mov r11, [rax+8]
    mov [rsp], r10
    mov [rsp+8], r11
    # El pequeño almacén de cajas reparte una página entre 256 lecturas. Así
    # cada acceso sigue devolviendo una dirección estable sin reservar una
    # página completa para solo dos palabras.
    call pf_dynamic_box_alloc
    mov r10, [rsp]
    mov r11, [rsp+8]
    add rsp, 24
    mov [rax], r10
    mov [rax+8], r11
    ret

# Cambia lista[índice] y devuelve la referencia original para la pila IR.
pf_list_set:
    mov ecx, 1
    jmp pf_list_set_tagged

# Entrada: rdi=lista, rsi=índice, rdx=valor, rcx=etiqueta.
pf_list_set_tagged:
    call pf_list_slot
    mov [rax], rcx
    mov [rax+8], rdx
    mov rax, rdi
    ret

# Entrada dinámica: rdx apunta a {etiqueta, contenido}.
pf_list_set_dynamic:
    mov rcx, [rdx]
    mov rdx, [rdx+8]
    jmp pf_list_set_tagged

# Busca una clave interna por comparación exacta. Cero indica que no existe.
pf_dict_find:
    mov rax, [rdi+16]
    mov rcx, [rdi]
.L_dict_search:
    test rcx, rcx
    jz .L_dict_missing
    cmp [rax], rsi
    je .L_dict_found
    add rax, 16
    dec rcx
    jmp .L_dict_search
.L_dict_missing:
    xor eax, eax
.L_dict_found:
    ret

# Una clave inexistente produce un diagnóstico y salida 1, no memoria arbitraria.
pf_dict_get:
    call pf_dict_find
    test rax, rax
    jz pf_col_key_error
    mov rax, [rax+8]
    ret

# Actualiza una clave existente o añade un nuevo par conservando orden de inserción.
pf_dict_set:
    call pf_dict_find
    test rax, rax
    jz pf_col_append
    mov [rax+8], rdx
    mov rax, rdi
    ret

# Diagnósticos de ejecución en español, escritos en stderr.
pf_col_index_error:
    lea rsi, [rip+.L_index_message]
    mov edx, OFFSET FLAT:.L_index_length
    jmp pf_col_fail
pf_col_key_error:
    lea rsi, [rip+.L_key_message]
    mov edx, OFFSET FLAT:.L_key_length
    jmp pf_col_fail
pf_col_memory_error:
    lea rsi, [rip+.L_memory_message]
    mov edx, OFFSET FLAT:.L_memory_length
pf_col_fail:
    mov edi, 2
    mov eax, 1
    syscall
    mov edi, 1
    mov eax, 60
    syscall
.section .rodata
.L_index_message: .ascii "Error ST: indice fuera de rango\n"
.L_index_length = . - .L_index_message
.L_key_message: .ascii "Error ST: clave inexistente\n"
.L_key_length = . - .L_key_message
.L_memory_message: .ascii "Error ST: no se pudo reservar memoria (limite nativo: 256 MiB)\n"
.L_memory_length = . - .L_memory_message
.section .text

# Inserta un entero en un conjunto usando el mismo entero como clave y valor.
# pf_dict_set conserva una sola entrada por clave, incluso para valores negativos.
# Comparte el crecimiento y la referencia estable de las colecciones existentes.
pf_set_add:
    mov rdx, rsi
    jmp pf_dict_set

# Estado privado del ejecutable monohilo: contabilidad de reservas y límite de pila.
.section .bss
.balign 8
pf_heap_used: .zero 8
pf_stack_floor: .zero 8
.section .text
pf_stack_error:
    lea rsi, [rip+.L_stack_error]
    mov edx, OFFSET FLAT:.L_stack_error_size
    jmp pf_col_fail
.section .rodata
.L_stack_error: .ascii "Error ST: limite de pila o recursion excedido\n"
.L_stack_error_size = . - .L_stack_error
.section .text
